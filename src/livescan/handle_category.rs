use std::path::PathBuf;

use sea_orm::{ActiveModelTrait, Set};
use tracing::info;
use uuid::Uuid;

use crate::{
    livescan::scan_category::scan_category,
    models::{self, metadata::CategoryMetadata},
};

use super::{scan_library::ScannedCategory, Scanner};

impl Scanner {
    pub async fn handle_category(
        &self,
        category: &ScannedCategory,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!("category: {}", category.path.to_string_lossy());
        let titles = scan_category(&category.path).await;

        /* pre-cleanup to make sure there's no residual temp category */
        self.cleanup_temp_category(category);

        /* #region - category metadata */
        let mut category_metadata_path = category.path.clone();
        category_metadata_path.set_extension("toml");
        let mut category_metadata = CategoryMetadata::default();
        category_metadata
            .read_from_file(&category_metadata_path)
            .await;
        /* #endregion */

        /* #region - category thumbnail */
        let category_thumbnail = self
            .handle_thumbnail(category_metadata.thumbnail, &category.path)
            .await
            .map(|thumbnail| {
                let _ = thumbnail
                    .full_path
                    .strip_prefix(&self.app_state.env.library_path)
                    .unwrap()
                    .to_path_buf();
                thumbnail
            });

        let category_id = Uuid::new_v4().to_string();
        /* #endregion */

        /* #region - push thumbnail to DB */
        if let Some(thumbnail) = &category_thumbnail {
            info!(
                "thumbnail of {}: {}",
                &category.name,
                thumbnail.full_path.to_string_lossy()
            );
            let active_thumbnail = models::thumbnails::ActiveModel {
                id: Set(category_id.clone()),
                path: Set(thumbnail.full_path.to_string_lossy().into_owned()),
                blurhash: Set(thumbnail.blurhash_result.blurhash.clone()),
                width: Set(thumbnail.blurhash_result.width),
                height: Set(thumbnail.blurhash_result.height),
            };
            let _ = active_thumbnail.insert(&self.app_state.db).await?;
        } else {
            info!("thumbnail of {}: none", &category.name);
        }
        /* #endregion */

        /* #region - category name */
        let category_name = match category_metadata.title {
            Some(title) => title,
            None => category
                .path
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string(),
        };
        /* #endregion */

        /* #region - push to DB */
        let active_category = models::categories::ActiveModel {
            id: Set(category_id.clone()),
            name: Set(category_name),
            description: Set(category_metadata.description),
        };
        let _ = active_category.insert(&self.app_state.db).await?;
        /* #endregion */

        /* #region - titles */
        for title in titles {
            let _ = self
                .handle_title(&title, category, category_id.clone())
                .await;
        }
        /* #endregion */

        /* cleanup */
        self.cleanup_temp_category(category);

        Ok(())
    }
    fn cleanup_temp_category(&self, category: &ScannedCategory) {
        let mut temp_dir_category: PathBuf = PathBuf::from(&self.temp_dir);
        temp_dir_category.push(&category.name);
        let handle = tokio::spawn(async move {
            let _ = tokio::fs::remove_dir_all(&temp_dir_category).await;
        });
        std::mem::drop(handle);
    }
}
