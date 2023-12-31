use sea_orm::{ColumnTrait, Condition, EntityTrait, QueryFilter};

use super::{
    scan_library::{scan_library, ScannedCategory},
    Blurhash,
};
use crate::{models::prelude::*, AppState};
use std::{path::PathBuf, sync::Arc};

pub struct Scanner {
    pub(super) app_state: Arc<AppState>,
    pub(super) temp_dir: PathBuf,
    pub(super) blurhash: Blurhash,
    pub(super) categories: Vec<ScannedCategory>,
}

impl Scanner {
    pub async fn default(app_state: Arc<AppState>) -> Self {
        let app_state = Arc::clone(&app_state);
        let temp_dir = PathBuf::from(&app_state.env.temp_dir.clone());
        let ffmpeg_path = app_state.env.ffmpeg_path.clone();
        let djxl_path = app_state.env.djxl_path.clone();
        let ffmpeg_log_path = app_state.env.ffmpeg_log_path.clone();
        let categories = scan_library(&app_state.env.library_path).await;
        Self {
            app_state,
            temp_dir,
            blurhash: Blurhash {
                ffmpeg_path,
                djxl_path,
                ffmpeg_log_path,
            },
            categories,
        }
    }
    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut category_ids = Vec::new();

        for category in &self.categories {
            category_ids.push(self.handle_category(category).await?);
        }

        let mut condition = Condition::all();
        for id in &category_ids {
            condition = condition.add(categories::Column::Id.ne(id));
        }

        let _ = Categories::delete_many()
            .filter(condition)
            .exec(&self.app_state.db)
            .await?;

        let titles = Titles::find().all(&self.app_state.db).await?.into_iter();
        for title in titles {
            if !PathBuf::from(&title.path).exists() {
                let _ = Titles::delete_by_id(&title.id)
                    .exec(&self.app_state.db)
                    .await?;
            }
        }

        tracing::debug!("✅ finished scanning library");

        Ok(())
    }
}
