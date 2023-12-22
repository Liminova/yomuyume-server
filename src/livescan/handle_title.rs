use super::handle_thumbnail::HandleThumbnailResult;
use super::BlurhashResult;
use super::Scanner;
use crate::{
    livescan::scan_extracted::scan_extracted,
    models::{metadata::TitleMetadata, thumbnails, titles},
};
use rayon::prelude::*;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use std::{
    fs::File,
    path::{Path, PathBuf},
};
use tracing::{debug, error, info, warn};
use uuid::Uuid;
use zip::ZipArchive;

use fasthash::murmur3;

use super::{scan_category::ScannedTitle, scan_library::ScannedCategory};

impl Scanner {
    pub async fn handle_title(
        &self,
        title: &ScannedTitle,
        category: &ScannedCategory,
        category_id: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        /* #region - metadata */
        let mut title_metadata_path = title.path.clone();
        title_metadata_path.set_extension("toml");
        let mut title_metadata = TitleMetadata::default();
        title_metadata.read_from_file(&title_metadata_path).await;
        /* #endregion */

        /* #region - title name */
        let title_name = match title_metadata.title.clone() {
            Some(title) => title,
            None => title
                .path
                .file_stem()
                .ok_or_else(|| {
                    error!("error getting title name");
                    "error getting title name"
                })?
                .to_string_lossy()
                .to_string(),
        };
        /* #endregion */

        /* #region - check if title exist + gen id */
        let current_title_hash = Self::hash(&title.path).await?;
        let mut title_exist_in_db = true;
        let mut title_id = String::new();

        match titles::Entity::find()
            .filter(titles::Column::Path.eq(title.path_lossy()))
            .one(&self.app_state.db)
            .await
        {
            Ok(Some(found_title_in_db)) => {
                let old_title_hash = found_title_in_db.hash.clone();
                title_id = found_title_in_db.id.clone();

                // Update title metadata nontheless
                let mut active_title: titles::ActiveModel = found_title_in_db.into();
                active_title.title = Set(title_name.clone());
                active_title.category_id = Set(category_id.clone());
                active_title.description = Set(title_metadata.description.clone());
                active_title.author = Set(title_metadata.author.clone());
                active_title.release_date = Set(title_metadata.release_date.clone());

                if old_title_hash == current_title_hash {
                    info!("title already exist: {}", title_name);
                    return Ok(());
                } else {
                    active_title.date_updated = Set(chrono::Utc::now().timestamp().to_string());
                }

                let _ = active_title.update(&self.app_state.db).await.map_err(|e| {
                    error!("error updating metadata for title: {}\n", e);
                    e
                })?;
            }
            Err(e) => {
                error!("database error checking existance of title: {}\n", e);
                return Err(e.into());
            }
            Ok(None) => title_exist_in_db = false,
        }
        /* #endregion */

        /* #region - create title in DB if needed */
        if !title_exist_in_db {
            title_id = Uuid::new_v4().to_string();
            let now = chrono::Utc::now().timestamp().to_string();
            let active_title = titles::ActiveModel {
                id: Set(title_id.clone()),
                category_id: Set(category_id),
                description: Set(title_metadata.description.clone()),
                title: Set(title_name),
                author: Set(title_metadata.author.clone()),
                release_date: Set(title_metadata.release_date.clone()),
                path: Set(title.path_lossy()),
                hash: Set(current_title_hash),
                date_added: Set(now.clone()),
                date_updated: Set(now),
            };
            let _ = active_title.insert(&self.app_state.db).await.map_err(|e| {
                error!("error inserting title: {}\n", e);
                e
            })?;
        }
        /* #endregion */

        /* #region - read & extract zip */
        let title_file = File::open(&title.path).map_err(|e| {
            error!("error openning title: {}\n", e);
            e
        })?;
        let mut zip_archive = ZipArchive::new(&title_file).map_err(|e| {
            error!("error reading title: {}\n", e);
            e
        })?;

        let mut title_temp_dir = PathBuf::from(&self.temp_dir);
        title_temp_dir.push(&category.name);
        title_temp_dir.push(&title.name);
        zip_archive.extract(&title_temp_dir).map_err(|e| {
            error!(
                "error extracting {} to {}: {}\n",
                title.path.to_string_lossy(),
                title_temp_dir.to_string_lossy(),
                e
            );
            e
        })?;
        /* #endregion */

        /* #region - title thumbnail */
        let title_thumbnail = self
            .handle_thumbnail(title_metadata.thumbnail.clone(), &title_temp_dir)
            .await
            .map(|thumbnail| {
                let mut thumbnail_path = PathBuf::from(&thumbnail.full_path);

                let mut temp_dir_category: PathBuf = PathBuf::from(&self.temp_dir);
                temp_dir_category.push(&category.name);
                let mut temp_dir_title: PathBuf = PathBuf::from(&self.temp_dir);
                temp_dir_title.push(&category.name);
                temp_dir_title.push(&title.name);

                debug!("temp_dir_category: {}", temp_dir_category.to_string_lossy());
                debug!(
                    "thumbnail_full_path: {}",
                    thumbnail.full_path.to_string_lossy()
                );

                thumbnail_path = match thumbnail_path.strip_prefix(&temp_dir_title) {
                    Ok(thumbnail_path) => thumbnail_path.to_path_buf(),
                    Err(_) => PathBuf::from(""),
                };

                HandleThumbnailResult {
                    blurhash_result: thumbnail.blurhash_result,
                    full_path: thumbnail_path,
                }
            });
        /* #endregion */

        /* #region - push thumbnail to DB */
        if let Some(thumbnail) = &title_thumbnail {
            info!(
                "thumbnail of {}: {}",
                &title.name,
                thumbnail.full_path.to_string_lossy()
            );
            let active_thumbnail = thumbnails::ActiveModel {
                id: Set(title_id.clone()),
                path: Set(thumbnail.blurhash_result.filename.clone()),
                blurhash: Set(thumbnail.blurhash_result.blurhash.clone()),
                width: Set(thumbnail.blurhash_result.width),
                height: Set(thumbnail.blurhash_result.height),
            };
            let _ = active_thumbnail
                .insert(&self.app_state.db)
                .await
                .map_err(|e| {
                    error!("error inserting thumbnail: {}\n", e);
                    e
                })?;
        } else {
            warn!("no thumbnail found for {}", &title.name);
        }
        /* #endregion */

        /* #region - process pages */
        let pages = scan_extracted(&title_temp_dir, &self.image_formats).await;

        #[cfg(debug_assertions)]
        let start_blurhash_encode = std::time::Instant::now();

        let blurhashes: Vec<BlurhashResult> = pages
            .par_iter()
            .filter_map(|page| self.blurhash.encode(&page.path, &page.ext))
            .collect();

        #[cfg(debug_assertions)]
        {
            let elapsed = start_blurhash_encode.elapsed();
            let average = elapsed / blurhashes.len() as u32;
            tracing::debug!(
                "blurhash encode: {} pages, elapsed: {:?}, average: {:?}",
                blurhashes.len(),
                elapsed,
                average
            );
        };
        /* #endregion */

        /* #region - push page to DB */
        let mut active_pages = Vec::new();
        for blurhash in blurhashes {
            let active_page = crate::models::pages::ActiveModel {
                id: Set(Uuid::new_v4().to_string()),
                title_id: Set(title_id.clone()),
                path: Set(blurhash.filename.clone()),
                hash: Set(blurhash.blurhash),
                width: Set(blurhash.width),
                height: Set(blurhash.height),
                description: Set(title_metadata.get_page_desc(blurhash.filename)),
            };
            active_pages.push(active_page);
        }
        let _ = crate::models::pages::Entity::insert_many(active_pages)
            .exec(&self.app_state.db)
            .await
            .map_err(|e| {
                error!("error inserting pages: {}\n", e);
                e
            })?;
        /* #endregion */

        /* #region - cleanup */
        tokio::fs::remove_dir_all(&title_temp_dir)
            .await
            .map_err(|e| {
                warn!("error removing temp dir: {}\n", e);
                e
            })?;
        /* #endregion */

        Ok(())
    }

    async fn hash(path: &Path) -> Result<String, Box<dyn std::error::Error>> {
        let content = tokio::fs::read(path).await?;
        Ok(murmur3::hash128(content).to_string())
    }
}
