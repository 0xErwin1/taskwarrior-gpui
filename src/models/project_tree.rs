use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ProjectNode {
    pub name: String,
    pub full_path: String,
    pub task_count: usize,
    pub direct_task_count: usize,
    pub level: usize,
    pub children_indices: Vec<usize>,
    pub is_expanded: bool,
}

impl ProjectNode {
    pub fn new(name: String, full_path: String, level: usize) -> Self {
        Self {
            name,
            full_path,
            task_count: 0,
            direct_task_count: 0,
            level,
            children_indices: Vec::new(),
            is_expanded: false,
        }
    }

    #[inline]
    pub fn has_children(&self) -> bool {
        !self.children_indices.is_empty()
    }
}

#[derive(Debug, Clone)]
pub struct ProjectTree {
    nodes: Vec<ProjectNode>,
    root_indices: Vec<usize>,
    path_to_index: HashMap<String, usize>,
    expanded_paths: HashMap<String, bool>,
}

impl ProjectTree {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            root_indices: Vec::new(),
            path_to_index: HashMap::new(),
            expanded_paths: HashMap::new(),
        }
    }

    pub fn build_from_projects(&mut self, projects: &[(String, usize)]) {
        self.nodes.clear();
        self.root_indices.clear();
        self.path_to_index.clear();

        for (project_path, task_count) in projects {
            if project_path.is_empty() {
                continue;
            }

            let segments: Vec<&str> = project_path.split('.').collect();
            self.insert_project(&segments, task_count);
        }

        self.root_indices.sort_by(|a, b| {
            self.nodes[*a]
                .name
                .to_lowercase()
                .cmp(&self.nodes[*b].name.to_lowercase())
        });

        let nodes_len = self.nodes.len();
        for i in 0..nodes_len {
            let mut indices_with_names: Vec<_> = self.nodes[i]
                .children_indices
                .iter()
                .map(|&idx| (idx, self.nodes[idx].name.to_lowercase()))
                .collect();
            indices_with_names.sort_by(|a, b| a.1.cmp(&b.1));
            self.nodes[i].children_indices =
                indices_with_names.into_iter().map(|(idx, _)| idx).collect();
        }

        for (path, is_expanded) in &self.expanded_paths {
            if let Some(&idx) = self.path_to_index.get(path) {
                self.nodes[idx].is_expanded = *is_expanded;
            }
        }
    }

    fn insert_project(&mut self, segments: &[&str], task_count: &usize) {
        if segments.is_empty() {
            return;
        }

        let mut current_path = String::new();
        let mut parent_idx: Option<usize> = None;

        for (level, &segment) in segments.iter().enumerate() {
            if !current_path.is_empty() {
                current_path.push('.');
            }
            current_path.push_str(segment);

            let node_idx = if let Some(&idx) = self.path_to_index.get(&current_path) {
                idx
            } else {
                let new_idx = self.nodes.len();
                let is_expanded = self
                    .expanded_paths
                    .get(&current_path)
                    .copied()
                    .unwrap_or(false);

                let node = ProjectNode {
                    name: segment.to_string(),
                    full_path: current_path.clone(),
                    task_count: 0,
                    direct_task_count: 0,
                    level,
                    children_indices: Vec::new(),
                    is_expanded,
                };

                self.nodes.push(node);
                self.path_to_index.insert(current_path.clone(), new_idx);

                if let Some(parent) = parent_idx {
                    self.nodes[parent].children_indices.push(new_idx);
                } else {
                    self.root_indices.push(new_idx);
                }

                new_idx
            };

            self.nodes[node_idx].task_count += task_count;

            if level == segments.len() - 1 {
                self.nodes[node_idx].direct_task_count += task_count;
            }

            parent_idx = Some(node_idx);
        }
    }

    pub fn toggle_expansion(&mut self, full_path: &str) {
        if let Some(&idx) = self.path_to_index.get(full_path) {
            let new_state = !self.nodes[idx].is_expanded;
            self.nodes[idx].is_expanded = new_state;
            self.expanded_paths.insert(full_path.to_string(), new_state);
        }
    }

    pub fn expand_path(&mut self, full_path: &str) {
        let segments: Vec<&str> = full_path.split('.').collect();
        let mut current_path = String::new();

        for segment in segments {
            if !current_path.is_empty() {
                current_path.push('.');
            }
            current_path.push_str(segment);

            if let Some(&idx) = self.path_to_index.get(&current_path) {
                self.nodes[idx].is_expanded = true;
                self.expanded_paths.insert(current_path.clone(), true);
            }
        }
    }

    pub fn collapse_all(&mut self) {
        for node in &mut self.nodes {
            node.is_expanded = false;
        }
        self.expanded_paths.clear();
    }

    pub fn root_indices(&self) -> &[usize] {
        &self.root_indices
    }

    pub fn get_node(&self, idx: usize) -> Option<&ProjectNode> {
        self.nodes.get(idx)
    }

    pub fn get_node_mut(&mut self, idx: usize) -> Option<&mut ProjectNode> {
        self.nodes.get_mut(idx)
    }

    pub fn find_by_path(&self, path: &str) -> Option<&ProjectNode> {
        self.path_to_index
            .get(path)
            .and_then(|&idx| self.nodes.get(idx))
    }

    pub fn iter_visible(&self) -> Vec<(usize, &ProjectNode)> {
        let mut result = Vec::new();
        for &root_idx in &self.root_indices {
            self.collect_visible(root_idx, &mut result);
        }
        result
    }

    fn collect_visible<'a>(&'a self, idx: usize, result: &mut Vec<(usize, &'a ProjectNode)>) {
        if let Some(node) = self.nodes.get(idx) {
            result.push((idx, node));

            if node.is_expanded {
                for &child_idx in &node.children_indices {
                    self.collect_visible(child_idx, result);
                }
            }
        }
    }
}

impl Default for ProjectTree {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_simple_tree() {
        let mut tree = ProjectTree::new();
        tree.build_from_projects(&[("Work".to_string(), 5), ("Home".to_string(), 3)]);

        assert_eq!(tree.root_indices().len(), 2);
    }

    #[test]
    fn test_build_nested_tree() {
        let mut tree = ProjectTree::new();
        tree.build_from_projects(&[
            ("Work.Backend.API".to_string(), 2),
            ("Work.Backend.DB".to_string(), 3),
            ("Work.Frontend".to_string(), 5),
        ]);

        let work_node = tree.find_by_path("Work").unwrap();
        assert_eq!(work_node.task_count, 10);
        assert_eq!(work_node.direct_task_count, 0);

        let backend_node = tree.find_by_path("Work.Backend").unwrap();
        assert_eq!(backend_node.task_count, 5);
    }

    #[test]
    fn test_toggle_expansion() {
        let mut tree = ProjectTree::new();
        tree.build_from_projects(&[("Work.Backend".to_string(), 5)]);

        assert!(!tree.find_by_path("Work").unwrap().is_expanded);

        tree.toggle_expansion("Work");
        assert!(tree.find_by_path("Work").unwrap().is_expanded);

        tree.toggle_expansion("Work");
        assert!(!tree.find_by_path("Work").unwrap().is_expanded);
    }
}
