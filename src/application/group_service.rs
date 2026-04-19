use crate::domain::group::{CreateGroup, GroupError, GroupMember, GroupRepository, GroupResponse, UpdateGroup};
use crate::domain::task::{PaginatedResponse, PaginationParams};
use uuid::Uuid;

pub struct GroupService {
    group_repository: Box<dyn GroupRepository>,
}

impl GroupService {
    pub fn new(group_repository: Box<dyn GroupRepository>) -> Self {
        Self { group_repository }
    }

    #[tracing::instrument(skip(self, create_group))]
    pub async fn create_group(&self, create_group: CreateGroup, user_id: Uuid) -> Result<GroupResponse, GroupError> {
        let group = self.group_repository.create(&create_group, user_id).await?;
        Ok(GroupResponse {
            id: group.id,
            name: group.name,
            created_at: group.created_at,
            created_by: group.created_by,
            updated_at: group.updated_at,
            updated_by: group.updated_by,
        })
    }

    #[tracing::instrument(skip(self, pagination))]
    pub async fn get_all_groups(
        &self,
        pagination: PaginationParams,
    ) -> Result<PaginatedResponse<GroupResponse>, GroupError> {
        let (groups, total_items) = self.group_repository.find_all(&pagination).await?;
        let page = pagination.page.unwrap_or(1);
        let limit = pagination.limit.unwrap_or(10);
        let total_pages = (total_items as f64 / limit as f64).ceil() as i64;

        let items = groups
            .into_iter()
            .map(|group| GroupResponse {
                id: group.id,
                name: group.name,
                created_at: group.created_at,
                created_by: group.created_by,
                updated_at: group.updated_at,
                updated_by: group.updated_by,
            })
            .collect();

        Ok(PaginatedResponse {
            items,
            total_items,
            page,
            limit,
            total_pages,
        })
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_group_by_id(&self, id: Uuid) -> Result<GroupResponse, GroupError> {
        let group = self
            .group_repository
            .find_by_id(id)
            .await?
            .ok_or(GroupError::GroupNotFound)?;
        Ok(GroupResponse {
            id: group.id,
            name: group.name,
            created_at: group.created_at,
            created_by: group.created_by,
            updated_at: group.updated_at,
            updated_by: group.updated_by,
        })
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_group_members(&self, group_id: Uuid) -> Result<Vec<GroupMember>, GroupError> {
        let members = self.group_repository.find_users_by_group_id(group_id).await?;
        Ok(members)
    }

    #[tracing::instrument(skip(self, update_group))]
    pub async fn update_group(
        &self,
        id: Uuid,
        update_group: UpdateGroup,
        user_id: Uuid,
    ) -> Result<GroupResponse, GroupError> {
        let group = self
            .group_repository
            .update(id, &update_group, user_id)
            .await?
            .ok_or(GroupError::GroupNotFound)?;
        Ok(GroupResponse {
            id: group.id,
            name: group.name,
            created_at: group.created_at,
            created_by: group.created_by,
            updated_at: group.updated_at,
            updated_by: group.updated_by,
        })
    }

    #[tracing::instrument(skip(self))]
    pub async fn delete_group(&self, id: Uuid) -> Result<(), GroupError> {
        let deleted = self.group_repository.delete(id).await?;
        if !deleted {
            return Err(GroupError::GroupNotFound);
        }
        Ok(())
    }
}
