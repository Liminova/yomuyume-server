use serde::Serialize;

#[derive(Serialize)]
pub struct Category {
    pub id: uuid::Uuid,
    pub name: String,
    pub description: Option<String>,
}
