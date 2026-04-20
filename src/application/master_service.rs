use crate::domain::master::{MasterError, MasterRepository, ProgressOption, RoleOption};

pub struct MasterService {
    master_repository: Box<dyn MasterRepository>,
}

impl MasterService {
    pub fn new(master_repository: Box<dyn MasterRepository>) -> Self {
        Self { master_repository }
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_progress_options(&self) -> Result<Vec<ProgressOption>, MasterError> {
        let options = self.master_repository.find_all_progress_options().await?;
        Ok(options)
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_role_options(&self) -> Result<Vec<RoleOption>, MasterError> {
        let options = self.master_repository.find_all_role_options().await?;
        Ok(options)
    }
}
