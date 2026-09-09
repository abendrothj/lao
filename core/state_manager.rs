use crate::path_policy::validate_identifier;
use crate::workflow_state::{WorkflowState, WorkflowStatus};
use std::collections::HashMap;
use std::fs;
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};

/// Workflow ids are interpolated into state file paths; reject anything that
/// could traverse outside the state directory.
fn checked_id(workflow_id: &str) -> std::io::Result<&str> {
    validate_identifier(workflow_id).map_err(|e| {
        Error::new(
            ErrorKind::InvalidInput,
            format!("invalid workflow id: {}", e),
        )
    })?;
    Ok(workflow_id)
}

pub struct WorkflowStateManager {
    state_dir: PathBuf,
    states: HashMap<String, WorkflowState>,
}

impl WorkflowStateManager {
    pub fn new<P: AsRef<Path>>(state_dir: P) -> std::io::Result<Self> {
        let state_dir = state_dir.as_ref().to_path_buf();
        fs::create_dir_all(&state_dir)?;

        let mut manager = Self {
            state_dir,
            states: HashMap::new(),
        };

        manager.load_all_states()?;
        Ok(manager)
    }

    pub fn save_state(&mut self, state: &WorkflowState) -> std::io::Result<()> {
        let file_path = self
            .state_dir
            .join(format!("{}.json", checked_id(&state.workflow_id)?));
        let json = serde_json::to_string_pretty(state)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        fs::write(file_path, json)?;
        self.states.insert(state.workflow_id.clone(), state.clone());
        Ok(())
    }

    pub fn load_state(&self, workflow_id: &str) -> std::io::Result<Option<WorkflowState>> {
        let file_path = self
            .state_dir
            .join(format!("{}.json", checked_id(workflow_id)?));
        if !file_path.exists() {
            return Ok(None);
        }

        let json = fs::read_to_string(file_path)?;
        let state: WorkflowState = serde_json::from_str(&json)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(Some(state))
    }

    pub fn delete_state(&mut self, workflow_id: &str) -> std::io::Result<()> {
        let file_path = self
            .state_dir
            .join(format!("{}.json", checked_id(workflow_id)?));
        if file_path.exists() {
            fs::remove_file(file_path)?;
        }
        self.states.remove(workflow_id);
        Ok(())
    }

    pub fn list_states(&self) -> Vec<&WorkflowState> {
        self.states.values().collect()
    }

    pub fn list_active_workflows(&self) -> Vec<&WorkflowState> {
        self.states
            .values()
            .filter(|state| {
                matches!(
                    state.status,
                    WorkflowStatus::Running | WorkflowStatus::Pending
                )
            })
            .collect()
    }

    pub fn list_scheduled_workflows(&self) -> Vec<&WorkflowState> {
        self.states
            .values()
            .filter(|state| matches!(state.status, WorkflowStatus::Scheduled))
            .collect()
    }

    fn load_all_states(&mut self) -> std::io::Result<()> {
        if !self.state_dir.exists() {
            return Ok(());
        }

        for entry in fs::read_dir(&self.state_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Some(filename) = path.file_stem().and_then(|s| s.to_str()) {
                    if let Ok(Some(state)) = self.load_state(filename) {
                        self.states.insert(state.workflow_id.clone(), state);
                    }
                }
            }
        }

        Ok(())
    }

    pub fn cleanup_old_states(&mut self, max_age_hours: u64) -> std::io::Result<usize> {
        let cutoff =
            std::time::SystemTime::now() - std::time::Duration::from_secs(max_age_hours * 3600);
        let mut removed_count = 0;

        let to_remove: Vec<String> = self
            .states
            .values()
            .filter(|state| {
                matches!(
                    state.status,
                    WorkflowStatus::Completed | WorkflowStatus::Failed | WorkflowStatus::Cancelled
                ) && state
                    .completed_at
                    .is_some_and(|completed| completed < cutoff)
            })
            .map(|state| state.workflow_id.clone())
            .collect();

        for workflow_id in to_remove {
            self.delete_state(&workflow_id)?;
            removed_count += 1;
        }

        Ok(removed_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workflow_state::WorkflowStatus;
    use std::time::{Duration, SystemTime};

    fn temp_state_dir() -> tempfile::TempDir {
        tempfile::tempdir().expect("create temp dir")
    }

    fn make_state(id: &str, status: WorkflowStatus) -> WorkflowState {
        let mut state = WorkflowState::new(id.to_string(), format!("Workflow {}", id), 2);
        state.status = status;
        state
    }

    #[test]
    fn test_create_and_load_state() {
        let dir = temp_state_dir();
        let mut manager = WorkflowStateManager::new(dir.path()).unwrap();

        let state = make_state("wf-1", WorkflowStatus::Running);
        manager.save_state(&state).unwrap();

        let loaded = manager.load_state("wf-1").unwrap();
        assert!(loaded.is_some());
        let loaded = loaded.unwrap();
        assert_eq!(loaded.workflow_id, "wf-1");
        assert!(matches!(loaded.status, WorkflowStatus::Running));
    }

    #[test]
    fn test_load_nonexistent_state() {
        let dir = temp_state_dir();
        let manager = WorkflowStateManager::new(dir.path()).unwrap();

        let loaded = manager.load_state("nonexistent").unwrap();
        assert!(loaded.is_none());
    }

    #[test]
    fn test_delete_state() {
        let dir = temp_state_dir();
        let mut manager = WorkflowStateManager::new(dir.path()).unwrap();

        let state = make_state("wf-del", WorkflowStatus::Completed);
        manager.save_state(&state).unwrap();
        assert_eq!(manager.list_states().len(), 1);

        manager.delete_state("wf-del").unwrap();
        assert_eq!(manager.list_states().len(), 0);
        assert!(manager.load_state("wf-del").unwrap().is_none());
    }

    #[test]
    fn test_delete_nonexistent_state_is_ok() {
        let dir = temp_state_dir();
        let mut manager = WorkflowStateManager::new(dir.path()).unwrap();
        // Should not error
        manager.delete_state("nope").unwrap();
    }

    #[test]
    fn test_list_states() {
        let dir = temp_state_dir();
        let mut manager = WorkflowStateManager::new(dir.path()).unwrap();

        manager
            .save_state(&make_state("a", WorkflowStatus::Pending))
            .unwrap();
        manager
            .save_state(&make_state("b", WorkflowStatus::Running))
            .unwrap();
        manager
            .save_state(&make_state("c", WorkflowStatus::Completed))
            .unwrap();

        assert_eq!(manager.list_states().len(), 3);
    }

    #[test]
    fn test_list_active_workflows() {
        let dir = temp_state_dir();
        let mut manager = WorkflowStateManager::new(dir.path()).unwrap();

        manager
            .save_state(&make_state("a", WorkflowStatus::Pending))
            .unwrap();
        manager
            .save_state(&make_state("b", WorkflowStatus::Running))
            .unwrap();
        manager
            .save_state(&make_state("c", WorkflowStatus::Completed))
            .unwrap();
        manager
            .save_state(&make_state("d", WorkflowStatus::Failed))
            .unwrap();

        let active = manager.list_active_workflows();
        assert_eq!(active.len(), 2);
    }

    #[test]
    fn test_list_scheduled_workflows() {
        let dir = temp_state_dir();
        let mut manager = WorkflowStateManager::new(dir.path()).unwrap();

        manager
            .save_state(&make_state("a", WorkflowStatus::Scheduled))
            .unwrap();
        manager
            .save_state(&make_state("b", WorkflowStatus::Running))
            .unwrap();

        let scheduled = manager.list_scheduled_workflows();
        assert_eq!(scheduled.len(), 1);
        assert_eq!(scheduled[0].workflow_id, "a");
    }

    #[test]
    fn test_persistence_across_instances() {
        let dir = temp_state_dir();

        // First instance saves state
        {
            let mut manager = WorkflowStateManager::new(dir.path()).unwrap();
            manager
                .save_state(&make_state("persist", WorkflowStatus::Running))
                .unwrap();
        }

        // Second instance should load it from disk
        {
            let manager = WorkflowStateManager::new(dir.path()).unwrap();
            assert_eq!(manager.list_states().len(), 1);
            assert_eq!(manager.list_states()[0].workflow_id, "persist");
        }
    }

    #[test]
    fn test_cleanup_old_states() {
        let dir = temp_state_dir();
        let mut manager = WorkflowStateManager::new(dir.path()).unwrap();

        // Create a completed state with old completed_at
        let mut old_state = make_state("old", WorkflowStatus::Completed);
        old_state.completed_at = Some(SystemTime::now() - Duration::from_secs(48 * 3600));
        manager.save_state(&old_state).unwrap();

        // Create a recent completed state
        let mut recent_state = make_state("recent", WorkflowStatus::Completed);
        recent_state.completed_at = Some(SystemTime::now());
        manager.save_state(&recent_state).unwrap();

        // Create a running state (should never be cleaned)
        manager
            .save_state(&make_state("running", WorkflowStatus::Running))
            .unwrap();

        let removed = manager.cleanup_old_states(24).unwrap();
        assert_eq!(removed, 1);
        assert_eq!(manager.list_states().len(), 2);
    }

    #[test]
    fn test_overwrite_existing_state() {
        let dir = temp_state_dir();
        let mut manager = WorkflowStateManager::new(dir.path()).unwrap();

        let mut state = make_state("wf-1", WorkflowStatus::Running);
        manager.save_state(&state).unwrap();

        state.status = WorkflowStatus::Completed;
        state.completed_at = Some(SystemTime::now());
        manager.save_state(&state).unwrap();

        let loaded = manager.load_state("wf-1").unwrap().unwrap();
        assert!(matches!(loaded.status, WorkflowStatus::Completed));
    }

    #[test]
    fn test_corrupt_json_file_skipped_on_load() {
        let dir = temp_state_dir();

        // Write corrupt JSON
        let corrupt_path = dir.path().join("bad.json");
        fs::write(&corrupt_path, "not valid json{{{").unwrap();

        // Should still construct successfully, just skip the corrupt file
        let manager = WorkflowStateManager::new(dir.path()).unwrap();
        assert_eq!(manager.list_states().len(), 0);
    }
}
