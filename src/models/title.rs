use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Deserialize, Serialize, ToSchema)]
pub struct Title {
    pub id: Uuid,
    pub title: String,
    pub category_id: Option<Uuid>,
    pub author: Option<String>,
    pub description: Option<String>,
    pub release_date: Option<DateTime<Utc>>,
    pub is_colored: Option<bool>,
    pub is_completed: Option<bool>,
    pub thumbnail: Option<String>,
}
