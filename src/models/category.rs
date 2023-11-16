use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct Category {
    pub id: uuid::Uuid,
    pub name: String,
    pub description: Option<String>,
}
