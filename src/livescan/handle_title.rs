use super::{
    scan_category::ScannedTitle, scan_library::ScannedCategory,
    thumbnail_finder::TitleThumbnailFinder, BlurhashResult, Scanner,
};
use crate::{
    constants::thumbnail_filestem,
    livescan::{scan_extracted::scan_extracted, thumbnail_finder::TitleThumbnailChangeFinder},
    models::{metadata::TitleMetadata, prelude::*},
};
use fasthash::murmur3;
use rayon::prelude::*;
use sea_orm::{ActiveModelTrait, ActiveValue::NotSet, ColumnTrait, EntityTrait, QueryFilter, Set};
use std::{
    fs::File,
    path::{Path, PathBuf},
};
use tracing::{debug, error, info, warn};
use uuid::Uuid;
use zip::ZipArchive;

impl Scanner {
    pub async fn handle_title(
        &self,
        title: &ScannedTitle,
        category: &ScannedCategory,
        category_id: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!("✅ found title: {}", title.path.to_string_lossy());

        /* #region - read <title>.toml */
        let mut title_metadata_path = title.path.clone();
        title_metadata_path.set_extension("toml");
        let mut title_metadata = TitleMetadata::from_file(&title_metadata_path).await;
        info!("- title (in metadata): {:?}", &title_metadata.title);
        info!("- description: {:?}", &title_metadata.description);
        info!("- thumbnail (in metadata): {:?}", &title_metadata.thumbnail);
        info!("- author: {:?}", &title_metadata.author);
        info!("- release_date: {:?}", &title_metadata.release_date);
        info!(
            "- tags count: {:?}",
            &title_metadata.tags.as_ref().map(|t| t.len())
        );
        info!(
            "- page description count: {:?}",
            &title_metadata.descriptions.as_ref().map(|d| d.len())
        );
        /* #endregion */

        /* #region - title defined in <title>.toml ? use it : use title file_stem */
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
        info!("- title (will use): {:?}", &title_name);
        /* #endregion */

        /* #region - check if title exist + gen uuid */
        let current_title_hash = Self::hash(&title.path).await.map_err(|e| {
            error!("error hashing: {}", e);
            e
        })?;
        let mut title_path_exist_in_db = false;
        let mut title_id = String::new();

        // By path -> hash change ? re-encode pages : update metadata -> return
        match Titles::find()
            .filter(titles::Column::Path.eq(title.path_lossy()))
            .one(&self.app_state.db)
            .await
        {
            Ok(Some(found_title_in_db)) => {
                title_path_exist_in_db = true;

                let old_title_hash = found_title_in_db.hash.clone();
                title_id = found_title_in_db.id.clone();

                // Update title metadata nonetheless
                let mut active_title: titles::ActiveModel = found_title_in_db.into();
                active_title.title = Set(title_name.clone());
                active_title.category_id = Set(category_id.clone());
                active_title.description = Set(title_metadata.description.clone());
                active_title.author = Set(title_metadata.author.clone());
                active_title.release_date = Set(title_metadata.release_date.clone());
                active_title.date_updated = Set(chrono::Utc::now().timestamp().to_string());

                // Hash changes -> re-encode pages
                let need_reencode = old_title_hash != current_title_hash;
                if need_reencode {
                    active_title.hash = Set(current_title_hash.clone());
                    active_title.date_updated = Set(chrono::Utc::now().timestamp().to_string());
                }
                let _ = active_title.update(&self.app_state.db).await.map_err(|e| {
                    error!("error update metadata in DB: {}", e);
                    e
                })?;

                // Thumbnail field in metadata != in DB -> re-encode thumbnail
                let thumbnail_in_db = Thumbnails::find()
                    .filter(thumbnails::Column::Id.eq(title_id.clone()))
                    .one(&self.app_state.db)
                    .await
                    .map_err(|e| {
                        error!("error search thumbnail in DB: {}", e);
                        e
                    })?;
                let thumbnail_in_db = thumbnail_in_db.map(|t| t.path);
                if thumbnail_in_db != title_metadata.thumbnail {
                    info!("thumbnail in DB != in metadata, re-encoding");
                    let _ = Thumbnails::delete_many()
                        .filter(thumbnails::Column::Id.eq(title_id.clone()))
                        .exec(&self.app_state.db)
                        .await
                        .map_err(|e| {
                            error!("error delete thumbnail in DB: {}", e);
                            e
                        })?;
                    let thumbnail = TitleThumbnailChangeFinder {
                        blurhash: &self.blurhash,
                        explicit_name: &title_metadata.thumbnail,
                        formats: &self.image_formats,
                        implicit_names: &thumbnail_filestem(),
                        temp_dir: &self.temp_dir,
                        title_path: &title.path,
                    }
                    .find()
                    .await
                    .ok_or_else(|| {
                        let err_msg = "error re-encoding new thumbnail";
                        error!(err_msg);
                        err_msg
                    })?;
                    // write BHResult -> <title>.toml and DB
                    title_metadata.set_thumbnail(thumbnail.filename.clone());
                    let _ = thumbnails::ActiveModel {
                        id: Set(title_id.clone()),
                        path: Set(thumbnail.filename),
                        blurhash: Set(thumbnail.blurhash),
                        width: Set(thumbnail.width),
                        height: Set(thumbnail.height),
                    }
                    .insert(&self.app_state.db)
                    .await
                    .map_err(|e| {
                        error!("error inserting thumbnail: {}", e);
                        e
                    })?;
                }

                if !need_reencode {
                    info!("found in DB by path, hash match, skipping");
                    return Ok(());
                }
                info!("found in DB by path, hash not match, finding hash");
            }
            Ok(None) => {
                info!("not found in DB by path, finding hash");
            }
            Err(e) => {
                error!("error search title in DB: {}", e);
                return Err(e.into());
            }
        }

        // By hash -> found match ? update metadata to match : encode -> return
        // Found match means nothing in the title.zip changed, so we can skip encoding pages
        match Titles::find()
            .filter(titles::Column::Hash.eq(current_title_hash.clone()))
            .one(&self.app_state.db)
            .await
        {
            Ok(Some(found_title_in_db)) => {
                info!("found in DB by hash, updating metadata and skipping encoding pages");

                let mut active_title: titles::ActiveModel = found_title_in_db.into();
                active_title.title = Set(title_name);
                active_title.category_id = Set(category_id.clone());
                active_title.description = Set(title_metadata.description.clone());
                active_title.author = Set(title_metadata.author.clone());
                active_title.release_date = Set(title_metadata.release_date.clone());
                active_title.date_updated = Set(chrono::Utc::now().timestamp().to_string());

                let _ = active_title.update(&self.app_state.db).await.map_err(|e| {
                    error!("error update metadata in DB: {}", e);
                    e
                })?;

                return Ok(()); // return this handle_title function
            }
            Ok(None) => {
                info!("not found in DB by hash, inserting title to DB and encoding pages");
            }
            Err(e) => {
                error!("error check if exist in DB: {}", e);
                return Err(e.into());
            }
        }
        /* #endregion */

        /* #region - create title in DB if find by hash + path failed */
        if !title_path_exist_in_db {
            title_id = Uuid::new_v4().to_string();
            let now = chrono::Utc::now().timestamp().to_string();
            let _ = titles::ActiveModel {
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
            }
            .insert(&self.app_state.db)
            .await
            .map_err(|e| {
                error!("error inserting title to DB: {}", e);
                e
            })?;
        }
        /* #endregion */

        /* #region - define temp dir, read zip, extract */
        let title_temp_dir = {
            let mut title_temp_dir = PathBuf::from(&self.temp_dir);
            title_temp_dir.push(&category.name);
            title_temp_dir.push(&title.name);
            title_temp_dir
        };
        let title_file = File::open(&title.path).map_err(|e| {
            error!("error openning title: {}", e);
            e
        })?;
        ZipArchive::new(title_file)
            .map_err(|e| {
                error!("error reading title: {}", e);
                e
            })?
            .extract(&title_temp_dir)
            .map_err(|e| {
                let temp_dir = title_temp_dir.to_string_lossy();
                error!("error extracting title to {}: {}", temp_dir, e);
                e
            })?;
        /* #endregion */

        /* #region - tags */
        let _ = tags::ActiveModel {
            id: NotSet,
            name: Set(title.name.clone()),
        }
        .insert(&self.app_state.db)
        .await
        .map_err(|e| {
            error!("error inserting tag: {}", e);
            e
        })?;
        let tag_id = tags::Entity::find()
            .filter(tags::Column::Name.eq(title.name.clone()))
            .one(&self.app_state.db)
            .await
            .map_err(|e| {
                error!("error finding tag: {}", e);
                e
            })?
            .ok_or_else(|| {
                let err_msg = "error finding tag";
                error!("{}", err_msg);
                err_msg
            })?
            .id;
        let _ = titles_tags::ActiveModel {
            id: NotSet,
            title_id: Set(title_id.clone()),
            tag_id: Set(tag_id),
        }
        .insert(&self.app_state.db)
        .await
        .map_err(|e| {
            error!("error inserting titles_tags: {}", e);
            e
        })?;
        /* #endregion */

        /* #region - process pages */
        let raw_pages = scan_extracted(&title_temp_dir, &self.image_formats).await;

        #[cfg(debug_assertions)]
        let start_blurhash_encode = std::time::Instant::now();

        let blurhashes: Vec<BlurhashResult> = raw_pages
            .par_iter()
            .filter_map(|page| self.blurhash.encode(&page.path, &page.ext))
            .collect();

        #[cfg(debug_assertions)]
        {
            let elapsed = start_blurhash_encode.elapsed();
            let average = elapsed / blurhashes.len() as u32;
            debug!(
                "blurhash encode: {} pages, elapsed: {:?}, average: {:?}",
                blurhashes.len(),
                elapsed,
                average
            );
        };
        /* #endregion */

        /* #region - clear old, push new page to DB */
        let deleted = Pages::delete_many()
            .filter(pages::Column::TitleId.eq(title_id.clone()))
            .exec(&self.app_state.db)
            .await
            .map_err(|e| {
                error!("error deleting old pages: {}", e);
                e
            })?;

        debug!("deleted {} pages", deleted.rows_affected);

        let mut active_pages = Vec::new();
        for blurhash in &blurhashes {
            let blurhash = blurhash.clone();
            let active_page = pages::ActiveModel {
                id: Set(Uuid::new_v4().to_string()),
                title_id: Set(title_id.clone()),
                path: Set(blurhash.filename.clone()),
                blurhash: Set(blurhash.blurhash),
                width: Set(blurhash.width),
                height: Set(blurhash.height),
                description: Set(title_metadata.get_page_desc(blurhash.filename)),
            };
            active_pages.push(active_page);
        }
        let _ = Pages::insert_many(active_pages)
            .exec(&self.app_state.db)
            .await
            .map_err(|e| {
                error!("error inserting pages: {}", e);
                e
            })?;
        /* #endregion */

        /* #region - title thumbnail */
        let _ = Thumbnails::delete_many()
            .filter(thumbnails::Column::Id.eq(title_id.clone()))
            .exec(&self.app_state.db)
            .await
            .map_err(|e| {
                error!("error deleting old thumbnail: {}", e);
                e
            })?;
        let possible_thumbnail_name = {
            let mut name = thumbnail_filestem();
            name.push(&title.name);
            if let Some(thumbnail) = &title_metadata.thumbnail {
                name.push(thumbnail);
            }
            name
        };
        let mut thumbnail_finder = TitleThumbnailFinder {
            blurhash_pages: &blurhashes,
            explicit_name: &title_metadata.thumbnail,
            implicit_names: &possible_thumbnail_name,
            formats: &self.image_formats,
            valid_pages: vec![],
        };
        if let Some(thumbnail) = thumbnail_finder.find() {
            info!("- thumbnail found: {}", thumbnail.filename);

            // write BHResult (filename) -> <title>.toml
            title_metadata.set_thumbnail(thumbnail.filename.clone());

            // write BHResult (everything) -> DB
            let _ = thumbnails::ActiveModel {
                id: Set(title_id),
                path: Set(thumbnail.filename),
                blurhash: Set(thumbnail.blurhash),
                width: Set(thumbnail.width),
                height: Set(thumbnail.height),
            }
            .insert(&self.app_state.db)
            .await
            .map_err(|e| {
                error!("error inserting thumbnail: {}", e);
                e
            })?;
        } else {
            // clear the thumbnail field in <title>.toml
            title_metadata.set_thumbnail(String::new());
            warn!("- no thumbnail found");
        }
        /* #endregion */

        /* #region - cleanup */
        let handle = tokio::spawn(async move {
            let _ = tokio::fs::remove_dir_all(&title_temp_dir).await;
        });
        std::mem::drop(handle);
        /* #endregion */

        Ok(())
    }

    async fn hash(path: &Path) -> Result<String, Box<dyn std::error::Error>> {
        let content = tokio::fs::read(path).await?;
        Ok(murmur3::hash128(content).to_string())
    }
}
