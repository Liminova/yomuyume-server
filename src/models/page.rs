use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, ToSchema)]
pub struct Page {
    pub id: Uuid,
    pub title_id: Uuid,
    pub path: String,
    pub hash: String,
    pub width: u32,
    pub height: u32,
}
