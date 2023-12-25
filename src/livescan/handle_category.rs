use super::{scan_library::ScannedCategory, Scanner};
use crate::{
    constants::thumbnail_filestem,
    livescan::{scan_category::scan_category, thumbnail_finder::CategoryThumbnailFinder},
    models::{metadata::CategoryMetadata, prelude::*},
};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use std::path::PathBuf;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

impl Scanner {
    pub async fn handle_category(
        &self,
        category: &ScannedCategory,
    ) -> Result<String, Box<dyn std::error::Error>> {
        info!("✅ found category: {}", category.path.to_string_lossy());

        /* pre-cleanup to make sure there's no residual temp category */
        self.cleanup_temp_category(category);

        /* #region read <possible_name>.toml */
        let mut category_metadata = CategoryMetadata::from(&category.path).await;
        info!("- name (in metadata): {:?}", &category_metadata.name);
        info!("- description: {:?}", &category_metadata.description);
        info!(
            "- thumbnail (in metadata): {:?}",
            &category_metadata.thumbnail
        );
        /* #endregion */

        /* #region - category's id = metadata || generated */
        let category_id = category_metadata.id.clone().map_or_else(
            || {
                let id = Uuid::new_v4().to_string();
                debug!("- id (generated): {}", &id);
                id
            },
            |id| {
                debug!("- id (in metadata): {}", &id);
                id
            },
        );
        category_metadata.set_id(category_id.clone());
        /* #endregion */

        /* #region - category's name = folder name || metadata */
        let category_name = match category_metadata.name {
            Some(name) => name,
            None => category
                .path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
        };
        info!("- name (will use): {:?}", &category_name);
        /* #endregion */

        /* #region - insert/update category info to DB */
        let category_exist_in_db = categories::Entity::find_by_id(&category_id)
            .one(&self.app_state.db)
            .await
            .map_err(|e| {
                error!("error search category in DB: {}", e);
                e
            })?;

        if let Some(category_exist_in_db) = category_exist_in_db {
            debug!("category already exists in DB");
            let mut active_category: categories::ActiveModel = category_exist_in_db.into();
            active_category.name = Set(category_name.clone());
            active_category.description = Set(category_metadata.description);
            let _ = active_category
                .update(&self.app_state.db)
                .await
                .map_err(|e| {
                    error!("error update category in DB: {}", e);
                    e
                })?;
        } else {
            debug!("category not exists in DB, insert");
            let _ = categories::ActiveModel {
                id: Set(category_id.clone()),
                name: Set(category_name),
                description: Set(category_metadata.description),
            }
            .insert(&self.app_state.db)
            .await
            .map_err(|e| {
                error!("error insert to DB: {}", e);
                e
            })?;
        }
        /* #endregion */

        /* #region - find category thumbnail */
        let mut implicit_thumbnail_names = thumbnail_filestem();
        implicit_thumbnail_names.push(&category.name);
        if let Some(thumbnail) = &category_metadata.thumbnail {
            implicit_thumbnail_names.push(thumbnail);
        }
        let mut category_thumbnail_finder = CategoryThumbnailFinder {
            parent_dir: &category.path,
            explicit_name: &category_metadata.thumbnail,
            implicit_names: &implicit_thumbnail_names,
            formats: &self.image_formats,
            blurhash: &self.blurhash,
        };
        /* #endregion */

        /* #region - push thumbnail to DB if needed */
        if let Some(thumbnail) = category_thumbnail_finder.find() {
            info!("- thumbnail found: {}", thumbnail.1.to_string_lossy());

            // check if exists in DB by blurhash
            let thumbnail_in_db = Thumbnails::find()
                .filter(thumbnails::Column::Blurhash.eq(&thumbnail.0.blurhash))
                .one(&self.app_state.db)
                .await
                .map_err(|e| {
                    error!("error search thumbnail in DB: {}", e);
                    e
                })?;

            // exists ? update path (blurhash same => dimensions same, no guarantee for path) : insert
            if let Some(thumbnail_in_db) = thumbnail_in_db {
                debug!("thumbnail already exists in DB");
                let mut active_thumbnail: thumbnails::ActiveModel = thumbnail_in_db.into();
                active_thumbnail.path = Set(thumbnail.1.to_string_lossy().into_owned());
                let _ = active_thumbnail
                    .update(&self.app_state.db)
                    .await
                    .map_err(|e| {
                        error!("error update thumbnail path in DB: {}", e);
                        e
                    })?;
            } else {
                debug!("thumbnail not exists in DB, insert");
                let _ = Thumbnails::delete_many()
                    .filter(thumbnails::Column::Id.eq(&category_id))
                    .exec(&self.app_state.db)
                    .await
                    .map_err(|e| {
                        error!("error delete thumbnail in DB: {}", e);
                        e
                    })?;
                let _ = thumbnails::ActiveModel {
                    id: Set(category_id.clone()),
                    path: Set(thumbnail.1.to_string_lossy().into_owned()),
                    blurhash: Set(thumbnail.0.blurhash),
                    width: Set(thumbnail.0.width),
                    height: Set(thumbnail.0.height),
                }
                .insert(&self.app_state.db)
                .await
                .map_err(|e| {
                    error!("error insert thumbnail to DB: {}", e);
                    e
                })?;
            }
        } else {
            warn!("- thumbnail not found");
        }
        /* #endregion */

        /* handle titles */
        for title in scan_category(&category.path).await {
            let _ = self
                .handle_title(&title, category, category_id.clone())
                .await;
        }

        /* cleanup */
        self.cleanup_temp_category(category);

        Ok(category_id)
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
