use chrono::{DateTime, Utc};

use super::repo_state::RepoState;
use crate::ports::{Program, ProgramRepository};

/// ProgramRepository fake.
#[derive(Clone)]
pub struct FakeProgramRepository(pub RepoState);

#[async_trait::async_trait]
impl ProgramRepository for FakeProgramRepository {
    async fn insert(
        &self,
        title: String,
        description: Option<String>,
        created_at: DateTime<Utc>,
    ) -> Result<Program, String> {
        let mut rows = self.0.programs.lock().unwrap();
        let id = RepoState::next_id(&rows, |program| program.id);
        let folder = format!(
            "{}-{}",
            title
                .to_lowercase()
                .replace(|c: char| !c.is_ascii_alphanumeric(), "-"),
            id
        );
        let program = Program {
            id,
            title,
            folder,
            description,
            cover_storage_key: None,
            category_id: None,
            created_at,
        };
        rows.push(program.clone());
        Ok(program)
    }

    async fn set_cover_key(&self, id: i64, key: &str) -> Result<bool, String> {
        let mut rows = self.0.programs.lock().unwrap();
        let Some(program) = rows.iter_mut().find(|program| program.id == id) else {
            return Ok(false);
        };
        program.cover_storage_key = Some(key.to_owned());
        Ok(true)
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<Program>, String> {
        Ok(self
            .0
            .programs
            .lock()
            .unwrap()
            .iter()
            .find(|program| program.id == id)
            .cloned())
    }

    async fn list_all(&self) -> Result<Vec<Program>, String> {
        Ok(self.0.programs.lock().unwrap().clone())
    }

    async fn update(
        &self,
        id: i64,
        title: String,
        description: Option<String>,
    ) -> Result<bool, String> {
        let mut rows = self.0.programs.lock().unwrap();
        let Some(program) = rows.iter_mut().find(|program| program.id == id) else {
            return Ok(false);
        };
        program.title = title;
        program.description = description;
        Ok(true)
    }

    async fn delete(&self, id: i64) -> Result<Option<Program>, String> {
        let mut rows = self.0.programs.lock().unwrap();
        let index = rows.iter().position(|program| program.id == id);
        Ok(index.map(|index| rows.remove(index)))
    }

    async fn set_category(&self, id: i64, category_id: Option<i64>) -> Result<bool, String> {
        let mut rows = self.0.programs.lock().unwrap();
        let Some(program) = rows.iter_mut().find(|program| program.id == id) else {
            return Ok(false);
        };
        program.category_id = category_id;
        Ok(true)
    }

    async fn clear_category(&self, category_id: i64) -> Result<u64, String> {
        let mut rows = self.0.programs.lock().unwrap();
        let mut cleared = 0;
        for program in rows.iter_mut() {
            if program.category_id == Some(category_id) {
                program.category_id = None;
                cleared += 1;
            }
        }
        Ok(cleared)
    }
}
