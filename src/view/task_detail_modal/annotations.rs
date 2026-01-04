use chrono::{DateTime, Utc};
use gpui::SharedString;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::task::TaskDetailVm;

pub(super) type AnnotationId = u64;

#[derive(Debug, Clone)]
pub(super) struct AnnotationState {
    pub(super) items: Vec<AnnotationView>,
    pub(super) draft: SharedString,
}

impl Default for AnnotationState {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            draft: SharedString::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub(super) struct AnnotationView {
    pub(super) id: AnnotationId,
    pub(super) created_at: DateTime<Utc>,
    pub(super) text: SharedString,
    pub(super) origin: AnnotationOrigin,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum AnnotationOrigin {
    Original,
    Added,
    Deleted,
    Modified {
        original_entry: DateTime<Utc>,
        original_text: String,
    },
}

fn annotation_id(entry: DateTime<Utc>, text: &str) -> AnnotationId {
    let mut hasher = DefaultHasher::new();
    entry.timestamp_millis().hash(&mut hasher);
    text.hash(&mut hasher);
    hasher.finish()
}

impl AnnotationState {
    pub(super) fn from_detail(detail: &TaskDetailVm) -> Self {
        let items: Vec<AnnotationView> = detail
            .annotations
            .iter()
            .map(|annotation| AnnotationView {
                id: annotation_id(annotation.entry, &annotation.content),
                created_at: annotation.entry,
                text: annotation.content.clone().into(),
                origin: AnnotationOrigin::Original,
            })
            .collect();

        Self {
            items,
            draft: SharedString::default(),
        }
    }

    pub(super) fn set_draft(&mut self, value: &str) {
        self.draft = value.to_string().into();
    }

    pub(super) fn clear_draft(&mut self) {
        self.draft = SharedString::default();
    }

    pub(super) fn add_local(&mut self, text: SharedString, created_at: DateTime<Utc>) {
        let id = annotation_id(created_at, text.as_ref());
        self.items.push(AnnotationView {
            id,
            created_at,
            text,
            origin: AnnotationOrigin::Added,
        });
    }

    pub(super) fn mark_deleted(&mut self, id: AnnotationId) -> Option<DateTime<Utc>> {
        let index = self.items.iter().position(|item| item.id == id)?;
        let item = &mut self.items[index];
        if item.origin == AnnotationOrigin::Added {
            self.items.remove(index);
            return None;
        }
        if item.origin == AnnotationOrigin::Deleted {
            return None;
        }
        item.origin = AnnotationOrigin::Deleted;
        Some(item.created_at)
    }

    pub(super) fn mark_modified(&mut self, id: AnnotationId) -> bool {
        if let Some(index) = self.items.iter().position(|item| item.id == id) {
            let item = &mut self.items[index];
            match item.origin {
                AnnotationOrigin::Original => {
                    item.origin = AnnotationOrigin::Modified {
                        original_entry: item.created_at,
                        original_text: item.text.to_string(),
                    };
                    return true;
                }
                _ => return false,
            }
        }
        false
    }
}
