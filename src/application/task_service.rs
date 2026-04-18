use crate::domain::task::{CreateTask, TaskError, TaskRepository, TaskResponse, UpdateTask};
use uuid::Uuid;

pub struct TaskService {
    task_repository: Box<dyn TaskRepository>,
}

impl TaskService {
    pub fn new(task_repository: Box<dyn TaskRepository>) -> Self {
        Self { task_repository }
    }

    #[tracing::instrument(skip(self, create_task))]
    pub async fn create_task(&self, create_task: CreateTask, id_user: Uuid) -> Result<TaskResponse, TaskError> {
        let task = self.task_repository.create(&create_task, id_user).await?;
        Ok(TaskResponse {
            id: task.id,
            task_name: task.task_name,
            description: task.description,
            id_user: task.id_user,
            created_at: task.created_at,
            updated_at: task.updated_at,
        })
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_all_tasks(&self) -> Result<Vec<TaskResponse>, TaskError> {
        let tasks = self.task_repository.find_all().await?;
        let responses = tasks
            .into_iter()
            .map(|task| TaskResponse {
                id: task.id,
                task_name: task.task_name,
                description: task.description,
                id_user: task.id_user,
                created_at: task.created_at,
                updated_at: task.updated_at,
            })
            .collect();
        Ok(responses)
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_task_by_id(&self, id: Uuid) -> Result<TaskResponse, TaskError> {
        let task = self
            .task_repository
            .find_by_id(id)
            .await?
            .ok_or(TaskError::TaskNotFound)?;
        Ok(TaskResponse {
            id: task.id,
            task_name: task.task_name,
            description: task.description,
            id_user: task.id_user,
            created_at: task.created_at,
            updated_at: task.updated_at,
        })
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_tasks_by_user(&self, id_user: Uuid) -> Result<Vec<TaskResponse>, TaskError> {
        let tasks = self.task_repository.find_by_user_id(id_user).await?;
        let responses = tasks
            .into_iter()
            .map(|task| TaskResponse {
                id: task.id,
                task_name: task.task_name,
                description: task.description,
                id_user: task.id_user,
                created_at: task.created_at,
                updated_at: task.updated_at,
            })
            .collect();
        Ok(responses)
    }

    #[tracing::instrument(skip(self))]
    pub async fn search_tasks(&self, id_user: Uuid, query: &str) -> Result<Vec<TaskResponse>, TaskError> {
        let tasks = self.task_repository.search(id_user, query).await?;
        let responses = tasks
            .into_iter()
            .map(|task| TaskResponse {
                id: task.id,
                task_name: task.task_name,
                description: task.description,
                id_user: task.id_user,
                created_at: task.created_at,
                updated_at: task.updated_at,
            })
            .collect();
        Ok(responses)
    }

    #[tracing::instrument(skip(self, update_task))]
    pub async fn update_task(
        &self,
        id: Uuid,
        update_task: UpdateTask,
    ) -> Result<TaskResponse, TaskError> {
        let task = self
            .task_repository
            .update(id, &update_task)
            .await?
            .ok_or(TaskError::TaskNotFound)?;
        Ok(TaskResponse {
            id: task.id,
            task_name: task.task_name,
            description: task.description,
            id_user: task.id_user,
            created_at: task.created_at,
            updated_at: task.updated_at,
        })
    }

    #[tracing::instrument(skip(self))]
    pub async fn delete_task(&self, id: Uuid) -> Result<(), TaskError> {
        let deleted = self.task_repository.delete(id).await?;
        if !deleted {
            return Err(TaskError::TaskNotFound);
        }
        Ok(())
    }
}
