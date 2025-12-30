use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use taskchampion::{
    Operations, Replica, ServerConfig, Status, StorageConfig, Tag, storage::AccessMode,
};
use uuid::Uuid;

use super::error::{TaskError, TaskResult};
use super::filter::TaskFilter;
use super::model::{Task, TaskDetailVm, TaskOverview, TaskStatus, TaskSummary};

pub struct TaskService {
    replica: Replica,
    taskdb_dir: PathBuf,
}

fn read_taskrc_config() -> TaskResult<PathBuf> {
    if let Ok(taskdata) = std::env::var("TASKDATA") {
        log::info!("Using TASKDATA env var: {}", taskdata);
        return Ok(PathBuf::from(taskdata));
    }

    let taskrc_path = if let Ok(taskrc) = std::env::var("TASKRC") {
        PathBuf::from(taskrc)
    } else {
        dirs::home_dir()
            .ok_or_else(|| TaskError::Config("Cannot find home directory".into()))?
            .join(".config/task/taskrc")
    };

    log::debug!("Looking for taskrc at: {:?}", taskrc_path);

    if taskrc_path.exists() {
        let content = std::fs::read_to_string(&taskrc_path)
            .map_err(|e| TaskError::Config(format!("Failed to read taskrc: {}", e)))?;

        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("data.location=") {
                let path = line.strip_prefix("data.location=").unwrap().trim();

                let expanded = if path.starts_with("~") {
                    let home = dirs::home_dir()
                        .ok_or_else(|| TaskError::Config("Cannot expand ~ in path".into()))?;
                    PathBuf::from(path.replacen("~", home.to_str().unwrap(), 1))
                } else {
                    PathBuf::from(path)
                };

                log::info!("Found data.location in taskrc: {:?}", expanded);
                return Ok(expanded);
            }
        }
    }

    log::warn!("No taskrc found or data.location not set, using default ~/.task");
    let default = dirs::home_dir()
        .ok_or_else(|| TaskError::Config("Cannot find home directory".into()))?
        .join(".task");
    Ok(default)
}

impl TaskService {
    pub fn new() -> TaskResult<Self> {
        let taskdb_dir = read_taskrc_config()?;

        log::debug!("TaskService: Using taskdb_dir: {:?}", taskdb_dir);
        log::debug!("TaskService: Directory exists: {}", taskdb_dir.exists());

        Self::with_path(taskdb_dir)
    }

    pub fn with_path(taskdb_dir: PathBuf) -> TaskResult<Self> {
        log::debug!("TaskService: Initializing with path: {:?}", taskdb_dir);

        let storage = StorageConfig::OnDisk {
            taskdb_dir: taskdb_dir.clone(),
            create_if_missing: true,
            access_mode: AccessMode::ReadWrite,
        };

        log::debug!("TaskService: Creating storage...");
        let replica = Replica::new(storage.into_storage().map_err(|e| {
            log::error!("TaskService: Storage creation failed: {}", e);
            TaskError::Storage(e.to_string())
        })?);

        log::debug!("TaskService: Replica created successfully");

        Ok(Self {
            replica,
            taskdb_dir,
        })
    }

    pub fn create_task(&mut self, description: String) -> TaskResult<Task> {
        let uuid = Uuid::new_v4();
        let mut ops = Operations::new();

        let mut tc_task = self
            .replica
            .create_task(uuid, &mut ops)
            .map_err(|e| TaskError::Storage(e.to_string()))?;

        tc_task
            .set_description(description, &mut ops)
            .map_err(|e| TaskError::Storage(e.to_string()))?;

        self.replica
            .commit_operations(ops)
            .map_err(|e| TaskError::Storage(e.to_string()))?;

        let working_set = self
            .replica
            .working_set()
            .map_err(|e| TaskError::Storage(e.to_string()))?;
        let mut task: Task = tc_task.into();
        task.working_id = working_set.by_uuid(uuid);

        Ok(task)
    }

    pub fn get_task(&mut self, uuid: Uuid) -> TaskResult<Option<Task>> {
        let tc_task = self
            .replica
            .get_task(uuid)
            .map_err(|e| TaskError::Storage(e.to_string()))?;

        match tc_task {
            None => Ok(None),
            Some(task) => {
                let working_set = self
                    .replica
                    .working_set()
                    .map_err(|e| TaskError::Storage(e.to_string()))?;
                let mut task: Task = task.into();
                task.working_id = working_set.by_uuid(uuid);
                Ok(Some(task))
            }
        }
    }

    pub fn get_task_detail(
        &mut self,
        uuid: Uuid,
        all_tasks: &[TaskSummary],
    ) -> TaskResult<TaskDetailVm> {
        let task = self.get_task(uuid)?.ok_or(TaskError::NotFound(uuid))?;
        Ok(TaskDetailVm::from_task(&task, all_tasks))
    }

    pub fn get_all_tasks(&mut self) -> TaskResult<Vec<Task>> {
        log::debug!("TaskService::get_all_tasks: Fetching all tasks from replica");
        let all = self.replica.all_tasks().map_err(|e| {
            log::error!("TaskService::get_all_tasks: Failed to get all tasks: {}", e);
            TaskError::Storage(e.to_string())
        })?;

        log::debug!(
            "TaskService::get_all_tasks: Found {} tasks in storage",
            all.len()
        );

        let working_set = self
            .replica
            .working_set()
            .map_err(|e| TaskError::Storage(e.to_string()))?;

        let tasks: Vec<Task> = all
            .iter()
            .map(|(uuid, tc_task)| {
                let mut task: Task = tc_task.clone().into();
                task.working_id = working_set.by_uuid(*uuid);
                task
            })
            .collect();

        log::debug!(
            "TaskService::get_all_tasks: Converted to {} Task objects",
            tasks.len()
        );
        Ok(tasks)
    }

    pub fn get_overview(&mut self) -> TaskResult<TaskOverview> {
        log::info!("TaskService::get_overview: Fetching complete overview");

        let tasks = self.get_all_tasks()?;

        log::debug!("Processing {} tasks for overview", tasks.len());

        let mut project_counts: HashMap<String, usize> = HashMap::new();
        let mut tag_counts: HashMap<String, usize> = HashMap::new();
        let mut pending_count = 0;
        let mut completed_count = 0;

        for task in &tasks {
            match task.status {
                TaskStatus::Pending => pending_count += 1,
                TaskStatus::Completed => completed_count += 1,
                _ => {}
            }

            if matches!(task.status, TaskStatus::Pending) {
                if let Some(project) = &task.project {
                    *project_counts.entry(project.clone()).or_insert(0) += 1;
                }

                for tag in &task.tags {
                    *tag_counts.entry(tag.clone()).or_insert(0) += 1;
                }
            }
        }

        let mut projects: Vec<(String, usize)> = project_counts.into_iter().collect();
        projects.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));

        let mut tags: Vec<(String, usize)> = tag_counts.into_iter().collect();
        tags.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));

        let total_tasks = tasks.len();

        log::info!(
            "Overview: {} tasks ({} pending, {} completed), {} projects, {} tags",
            total_tasks,
            pending_count,
            completed_count,
            projects.len(),
            tags.len()
        );

        Ok(TaskOverview {
            tasks,
            projects,
            tags,
            total_tasks,
            pending_tasks: pending_count,
            completed_tasks: completed_count,
        })
    }

    pub fn get_filtered_tasks(&mut self, filter: &TaskFilter) -> TaskResult<Vec<TaskSummary>> {
        let all = self.get_all_tasks()?;
        let summaries: Vec<TaskSummary> = all.iter().map(TaskSummary::from).collect();
        Ok(filter.apply(&summaries))
    }

    pub fn update_task(
        &mut self,
        uuid: Uuid,
        description: Option<String>,
        project: Option<Option<String>>,
        priority: Option<String>,
        tags: Option<HashSet<String>>,
        due: Option<Option<DateTime<Utc>>>,
        wait: Option<Option<DateTime<Utc>>>,
    ) -> TaskResult<Task> {
        let mut ops = Operations::new();

        let mut tc_task = self
            .replica
            .get_task(uuid)
            .map_err(|e| TaskError::Storage(e.to_string()))?
            .ok_or(TaskError::NotFound(uuid))?;

        if let Some(desc) = description {
            tc_task
                .set_description(desc, &mut ops)
                .map_err(|e| TaskError::Storage(e.to_string()))?;
        }

        if let Some(proj) = project {
            tc_task
                .set_value("project", proj, &mut ops)
                .map_err(|e| TaskError::Storage(e.to_string()))?;
        }

        if let Some(pri) = priority {
            tc_task
                .set_priority(pri, &mut ops)
                .map_err(|e| TaskError::Storage(e.to_string()))?;
        }

        if let Some(d) = due {
            tc_task
                .set_due(d, &mut ops)
                .map_err(|e| TaskError::Storage(e.to_string()))?;
        }

        if let Some(w) = wait {
            tc_task
                .set_wait(w, &mut ops)
                .map_err(|e| TaskError::Storage(e.to_string()))?;
        }

        if let Some(new_tags) = tags {
            let current_tags: HashSet<String> = tc_task.get_tags().map(|t| t.to_string()).collect();

            for tag_str in current_tags.difference(&new_tags) {
                if let Ok(tag) = Tag::try_from(tag_str.as_str()) {
                    tc_task
                        .remove_tag(&tag, &mut ops)
                        .map_err(|e| TaskError::Storage(e.to_string()))?;
                }
            }

            for tag_str in new_tags.difference(&current_tags) {
                if let Ok(tag) = Tag::try_from(tag_str.as_str()) {
                    tc_task
                        .add_tag(&tag, &mut ops)
                        .map_err(|e| TaskError::Storage(e.to_string()))?;
                }
            }
        }

        self.replica
            .commit_operations(ops)
            .map_err(|e| TaskError::Storage(e.to_string()))?;

        self.get_task(uuid)?.ok_or(TaskError::NotFound(uuid))
    }

    pub fn complete_task(&mut self, uuid: Uuid) -> TaskResult<Task> {
        let mut ops = Operations::new();

        let mut tc_task = self
            .replica
            .get_task(uuid)
            .map_err(|e| TaskError::Storage(e.to_string()))?
            .ok_or(TaskError::NotFound(uuid))?;

        tc_task
            .done(&mut ops)
            .map_err(|e| TaskError::Storage(e.to_string()))?;

        self.replica
            .commit_operations(ops)
            .map_err(|e| TaskError::Storage(e.to_string()))?;

        self.get_task(uuid)?.ok_or(TaskError::NotFound(uuid))
    }

    pub fn reopen_task(&mut self, uuid: Uuid) -> TaskResult<Task> {
        let mut ops = Operations::new();

        let mut tc_task = self
            .replica
            .get_task(uuid)
            .map_err(|e| TaskError::Storage(e.to_string()))?
            .ok_or(TaskError::NotFound(uuid))?;

        tc_task
            .set_status(Status::Pending, &mut ops)
            .map_err(|e| TaskError::Storage(e.to_string()))?;

        self.replica
            .commit_operations(ops)
            .map_err(|e| TaskError::Storage(e.to_string()))?;

        self.get_task(uuid)?.ok_or(TaskError::NotFound(uuid))
    }

    pub fn delete_task(&mut self, uuid: Uuid) -> TaskResult<()> {
        let mut ops = Operations::new();

        let mut tc_task = self
            .replica
            .get_task(uuid)
            .map_err(|e| TaskError::Storage(e.to_string()))?
            .ok_or(TaskError::NotFound(uuid))?;

        tc_task
            .set_status(Status::Deleted, &mut ops)
            .map_err(|e| TaskError::Storage(e.to_string()))?;

        self.replica
            .commit_operations(ops)
            .map_err(|e| TaskError::Storage(e.to_string()))?;

        Ok(())
    }

    pub fn start_task(&mut self, uuid: Uuid) -> TaskResult<Task> {
        let mut ops = Operations::new();

        let mut tc_task = self
            .replica
            .get_task(uuid)
            .map_err(|e| TaskError::Storage(e.to_string()))?
            .ok_or(TaskError::NotFound(uuid))?;

        tc_task
            .start(&mut ops)
            .map_err(|e| TaskError::Storage(e.to_string()))?;

        self.replica
            .commit_operations(ops)
            .map_err(|e| TaskError::Storage(e.to_string()))?;

        self.get_task(uuid)?.ok_or(TaskError::NotFound(uuid))
    }

    pub fn stop_task(&mut self, uuid: Uuid) -> TaskResult<Task> {
        let mut ops = Operations::new();

        let mut tc_task = self
            .replica
            .get_task(uuid)
            .map_err(|e| TaskError::Storage(e.to_string()))?
            .ok_or(TaskError::NotFound(uuid))?;

        tc_task
            .stop(&mut ops)
            .map_err(|e| TaskError::Storage(e.to_string()))?;

        self.replica
            .commit_operations(ops)
            .map_err(|e| TaskError::Storage(e.to_string()))?;

        self.get_task(uuid)?.ok_or(TaskError::NotFound(uuid))
    }

    pub fn add_tag(&mut self, uuid: Uuid, tag_str: &str) -> TaskResult<Task> {
        let mut ops = Operations::new();

        let mut tc_task = self
            .replica
            .get_task(uuid)
            .map_err(|e| TaskError::Storage(e.to_string()))?
            .ok_or(TaskError::NotFound(uuid))?;

        let tag = Tag::try_from(tag_str).map_err(|_| TaskError::InvalidTag(tag_str.to_string()))?;

        tc_task
            .add_tag(&tag, &mut ops)
            .map_err(|e| TaskError::Storage(e.to_string()))?;

        self.replica
            .commit_operations(ops)
            .map_err(|e| TaskError::Storage(e.to_string()))?;

        self.get_task(uuid)?.ok_or(TaskError::NotFound(uuid))
    }

    pub fn remove_tag(&mut self, uuid: Uuid, tag_str: &str) -> TaskResult<Task> {
        let mut ops = Operations::new();

        let mut tc_task = self
            .replica
            .get_task(uuid)
            .map_err(|e| TaskError::Storage(e.to_string()))?
            .ok_or(TaskError::NotFound(uuid))?;

        let tag = Tag::try_from(tag_str).map_err(|_| TaskError::InvalidTag(tag_str.to_string()))?;

        tc_task
            .remove_tag(&tag, &mut ops)
            .map_err(|e| TaskError::Storage(e.to_string()))?;

        self.replica
            .commit_operations(ops)
            .map_err(|e| TaskError::Storage(e.to_string()))?;

        self.get_task(uuid)?.ok_or(TaskError::NotFound(uuid))
    }

    pub fn list_tags(&mut self) -> TaskResult<Vec<(String, usize)>> {
        log::debug!("TaskService::list_tags: Getting all tasks");
        let all_tasks = self.get_all_tasks()?;
        log::debug!(
            "TaskService::list_tags: Processing {} tasks for tags",
            all_tasks.len()
        );

        let mut tag_counts: HashMap<String, usize> = HashMap::new();

        for task in all_tasks {
            if matches!(task.status, TaskStatus::Pending) {
                for tag in task.tags {
                    *tag_counts.entry(tag).or_insert(0) += 1;
                }
            }
        }

        log::debug!(
            "TaskService::list_tags: Found {} unique tags",
            tag_counts.len()
        );

        let mut stats: Vec<(String, usize)> = tag_counts.into_iter().collect();

        stats.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));

        Ok(stats)
    }

    pub fn list_projects(&mut self) -> TaskResult<Vec<(String, usize, usize)>> {
        log::debug!("TaskService::list_projects: Getting all tasks");
        let all_tasks = self.get_all_tasks()?;
        log::debug!(
            "TaskService::list_projects: Processing {} tasks for projects",
            all_tasks.len()
        );

        let mut project_counts: HashMap<String, (usize, usize)> = HashMap::new();

        for task in all_tasks {
            if let Some(project) = &task.project {
                let entry = project_counts.entry(project.clone()).or_insert((0, 0));
                entry.0 += 1;
                if matches!(task.status, TaskStatus::Pending) {
                    entry.1 += 1;
                }
            }
        }

        log::debug!(
            "TaskService::list_projects: Found {} unique projects",
            project_counts.len()
        );

        let stats: Vec<(String, usize, usize)> = project_counts
            .into_iter()
            .map(|(name, (task_count, pending_count))| (name, task_count, pending_count))
            .collect();

        Ok(stats)
    }

    pub fn get_projects_for_tree(&mut self) -> TaskResult<Vec<(String, usize)>> {
        log::debug!("TaskService::get_projects_for_tree: Getting project stats");
        let stats = self.list_projects()?;
        let result: Vec<(String, usize)> = stats
            .into_iter()
            .map(|(name, _total, pending)| (name, pending))
            .collect();
        log::debug!(
            "TaskService::get_projects_for_tree: Returning {} projects",
            result.len()
        );
        Ok(result)
    }

    pub fn add_annotation(&mut self, uuid: Uuid, description: String) -> TaskResult<Task> {
        let mut ops = Operations::new();

        let mut tc_task = self
            .replica
            .get_task(uuid)
            .map_err(|e| TaskError::Storage(e.to_string()))?
            .ok_or(TaskError::NotFound(uuid))?;

        let annotation = taskchampion::Annotation {
            entry: Utc::now(),
            description,
        };

        tc_task
            .add_annotation(annotation, &mut ops)
            .map_err(|e| TaskError::Storage(e.to_string()))?;

        self.replica
            .commit_operations(ops)
            .map_err(|e| TaskError::Storage(e.to_string()))?;

        self.get_task(uuid)?.ok_or(TaskError::NotFound(uuid))
    }

    pub fn remove_annotation(&mut self, uuid: Uuid, entry: DateTime<Utc>) -> TaskResult<Task> {
        let mut ops = Operations::new();

        let mut tc_task = self
            .replica
            .get_task(uuid)
            .map_err(|e| TaskError::Storage(e.to_string()))?
            .ok_or(TaskError::NotFound(uuid))?;

        tc_task
            .remove_annotation(entry, &mut ops)
            .map_err(|e| TaskError::Storage(e.to_string()))?;

        self.replica
            .commit_operations(ops)
            .map_err(|e| TaskError::Storage(e.to_string()))?;

        self.get_task(uuid)?.ok_or(TaskError::NotFound(uuid))
    }

    pub fn add_dependency(&mut self, uuid: Uuid, depends_on: Uuid) -> TaskResult<Task> {
        let mut ops = Operations::new();

        let mut tc_task = self
            .replica
            .get_task(uuid)
            .map_err(|e| TaskError::Storage(e.to_string()))?
            .ok_or(TaskError::NotFound(uuid))?;

        tc_task
            .add_dependency(depends_on, &mut ops)
            .map_err(|e| TaskError::Storage(e.to_string()))?;

        self.replica
            .commit_operations(ops)
            .map_err(|e| TaskError::Storage(e.to_string()))?;

        self.get_task(uuid)?.ok_or(TaskError::NotFound(uuid))
    }

    pub fn remove_dependency(&mut self, uuid: Uuid, depends_on: Uuid) -> TaskResult<Task> {
        let mut ops = Operations::new();

        let mut tc_task = self
            .replica
            .get_task(uuid)
            .map_err(|e| TaskError::Storage(e.to_string()))?
            .ok_or(TaskError::NotFound(uuid))?;

        tc_task
            .remove_dependency(depends_on, &mut ops)
            .map_err(|e| TaskError::Storage(e.to_string()))?;

        self.replica
            .commit_operations(ops)
            .map_err(|e| TaskError::Storage(e.to_string()))?;

        self.get_task(uuid)?.ok_or(TaskError::NotFound(uuid))
    }

    pub fn sync(&mut self) -> TaskResult<SyncResult> {
        let server_dir = self.taskdb_dir.join("server");

        if !server_dir.exists() {
            return Ok(SyncResult {
                success: false,
                message: "Server not configured".to_string(),
                local_ops_before: 0,
                local_ops_after: 0,
            });
        }

        let local_ops_before = self
            .replica
            .num_local_operations()
            .map_err(|e| TaskError::Storage(e.to_string()))?;

        let server_config = ServerConfig::Local { server_dir };

        let mut server = server_config
            .into_server()
            .map_err(|e| TaskError::Storage(e.to_string()))?;

        self.replica
            .sync(&mut server, false)
            .map_err(|e| TaskError::Storage(e.to_string()))?;

        let local_ops_after = self
            .replica
            .num_local_operations()
            .map_err(|e| TaskError::Storage(e.to_string()))?;

        Ok(SyncResult {
            success: true,
            message: "Sync completed".to_string(),
            local_ops_before,
            local_ops_after,
        })
    }

    pub fn pending_sync_operations(&mut self) -> TaskResult<usize> {
        Ok(self
            .replica
            .num_local_operations()
            .map_err(|e| TaskError::Storage(e.to_string()))?)
    }

    pub fn rebuild_working_set(&mut self, renumber: bool) -> TaskResult<()> {
        self.replica
            .rebuild_working_set(renumber)
            .map_err(|e| TaskError::Storage(e.to_string()))?;
        Ok(())
    }

    pub fn expire_tasks(&mut self) -> TaskResult<()> {
        self.replica
            .expire_tasks()
            .map_err(|e| TaskError::Storage(e.to_string()))?;
        Ok(())
    }

    pub fn working_set(&mut self) -> TaskResult<Vec<(usize, Task)>> {
        let ws = self
            .replica
            .working_set()
            .map_err(|e| TaskError::Storage(e.to_string()))?;
        let mut result = Vec::new();

        for (idx, uuid) in ws.iter() {
            if let Some(task) = self.get_task(uuid)? {
                result.push((idx, task));
            }
        }

        Ok(result)
    }

    pub fn get_task_by_working_id(&mut self, id: usize) -> TaskResult<Option<Task>> {
        let ws = self
            .replica
            .working_set()
            .map_err(|e| TaskError::Storage(e.to_string()))?;

        match ws.by_index(id) {
            None => Ok(None),
            Some(uuid) => self.get_task(uuid),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SyncResult {
    pub success: bool,
    pub message: String,
    pub local_ops_before: usize,
    pub local_ops_after: usize,
}
