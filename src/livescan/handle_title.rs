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
        let mut title_metadata = TitleMetadata::from(&{
            let mut title_metadata_path = title.path.clone();
            title_metadata_path.set_extension("toml");
            title_metadata_path
        })
        .await;

        debug!("metadata | [title] {:?} [desc] {:?} [thumb] {:?} [author] {:?} [release_date] {:?} [tags] {:?}",
            &title_metadata.title, &title_metadata.description, &title_metadata.thumbnail, &title_metadata.author, &title_metadata.release_date, &title_metadata.tags);
        /* #endregion */

        /* #region - title's name defined in <title>.toml ? use it : use title file_stem */
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
        debug!("title | {:?}", &title_name);
        /* #endregion */

        /* #region - check if title exist; gen uuid if needed; handle metadata changes */
        let title_hash_current = Self::hash(&title.path).await.map_err(|e| {
            error!("error hashing: {}", e);
            e
        })?;
        let mut title_path_exist_in_db = false;
        let mut title_id = String::new();

        // By path -> hash change ? by hash : update metadata -> return
        match Titles::find()
            .filter(titles::Column::Path.eq(title.path_lossy()))
            .one(&self.app_state.db)
            .await
        {
            Ok(Some(title_model)) => {
                title_path_exist_in_db = true;
                title_id = title_model.id.clone();

                tracing::debug!("hash in db: {}", title_model.hash);
                tracing::debug!("hash current: {}", title_hash_current);

                /* #region - update fields if metadata changed */
                let mut need_update = false;
                let mut active_title: titles::ActiveModel = Titles::find_by_id(&title_model.id)
                    .one(&self.app_state.db)
                    .await
                    .map_err(|e| {
                        error!("error search title in DB: {}", e);
                        e
                    })?
                    .ok_or_else(|| {
                        let err_msg = "error search title in DB, this should not happend";
                        error!("{}", err_msg);
                        err_msg
                    })?
                    .into();
                if title_model.title != title_name {
                    need_update = true;
                    active_title.title = Set(title_name.clone());
                }
                if title_model.category_id != category_id {
                    need_update = true;
                    active_title.category_id = Set(category_id.clone());
                }
                if title_model.description != title_metadata.description {
                    need_update = true;
                    active_title.description = Set(title_metadata.description.clone());
                }
                if title_model.author != title_metadata.author {
                    need_update = true;
                    active_title.author = Set(title_metadata.author.clone());
                }
                if title_model.release_date != title_metadata.release_date {
                    need_update = true;
                    active_title.release_date = Set(title_metadata.release_date.clone());
                }
                if title_model.hash != title_hash_current {
                    need_update = true;
                    active_title.date_updated = Set(chrono::Utc::now().timestamp().to_string());
                }
                if need_update {
                    let _ = active_title.update(&self.app_state.db).await.map_err(|e| {
                        error!("error update metadata in DB: {}", e);
                        e
                    })?;
                }
                /* #endregion */

                /* #region - update thumbnail if metadata changed */
                let thumbnail_path_in_db = Thumbnails::find()
                    .filter(thumbnails::Column::Id.eq(&title_id))
                    .one(&self.app_state.db)
                    .await
                    .map_err(|e| {
                        error!("error search thumbnail in DB: {}", e);
                        e
                    })?
                    .map(|t| t.path);
                if thumbnail_path_in_db != title_metadata.thumbnail {
                    info!("thumbnail in DB != in metadata, re-encoding");
                    let _ = Thumbnails::delete_many()
                        .filter(thumbnails::Column::Id.eq(&title_id))
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
                    title_metadata.set_thumbnail(thumbnail.file_name.clone());
                    let _ = thumbnails::ActiveModel {
                        id: Set(title_id.clone()),
                        path: Set(thumbnail.file_name),
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
                /* #endregion */

                /* #region - update pages' descs if metadata changed */
                let page_models = Pages::find()
                    .filter(pages::Column::TitleId.eq(&title_id))
                    .all(&self.app_state.db)
                    .await
                    .map_err(|e| {
                        error!("error search pages in DB: {}", e);
                        e
                    })?;
                for page in page_models {
                    let page_desc_metadata = title_metadata.get_page_desc(page.path.clone());
                    if page.description != page_desc_metadata {
                        let mut active_page: pages::ActiveModel = page.into();
                        active_page.description = Set(page_desc_metadata);
                        let _ = active_page.update(&self.app_state.db).await.map_err(|e| {
                            error!("error update page in DB: {}", e);
                            e
                        })?;
                    }
                }
                /* #endregion */

                if title_model.hash == title_hash_current {
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
            .filter(titles::Column::Hash.eq(&title_hash_current))
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

        /* #region - create if title is new, else update hash */
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
                hash: Set(title_hash_current),
                date_added: Set(now.clone()),
                date_updated: Set(now),
            }
            .insert(&self.app_state.db)
            .await
            .map_err(|e| {
                error!("error inserting title to DB: {}", e);
                e
            })?;
        } else {
            let active_title = Titles::find_by_id(&title_id)
                .one(&self.app_state.db)
                .await
                .map_err(|e| {
                    error!("error search title in DB: {}", e);
                    e
                })?
                .ok_or_else(|| {
                    let err_msg = "error search title in DB, this should not happend";
                    error!("{}", err_msg);
                    err_msg
                })?;

            let mut active_title: titles::ActiveModel = active_title.into();
            active_title.hash = Set(title_hash_current);

            let _ = active_title.update(&self.app_state.db).await.map_err(|e| {
                error!("error update hash in DB: {}", e);
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
        if let Some(tags) = title_metadata.tags.clone() {
            for tag in tags {
                // Get the tag_id
                let tag_id = {
                    // find the tag_name in DB first
                    let tag_in_db = tags::Entity::find()
                        .filter(tags::Column::Name.eq(&tag))
                        .one(&self.app_state.db)
                        .await
                        .map_err(|e| {
                            error!("error finding tag: {}", e);
                            e
                        })?;

                    // if found, get the id
                    if let Some(tag) = tag_in_db {
                        tag.id
                    } else {
                        // else, insert the tag_name to DB, get the id
                        let _ = tags::ActiveModel {
                            id: NotSet,
                            name: Set(tag.clone()),
                        }
                        .insert(&self.app_state.db)
                        .await
                        .map_err(|e| {
                            error!("error inserting tag: {}", e);
                            e
                        })?;
                        tags::Entity::find()
                            .filter(tags::Column::Name.eq(&tag))
                            .one(&self.app_state.db)
                            .await
                            .map_err(|e| {
                                error!("error finding tag: {}", e);
                                e
                            })?
                            .ok_or_else(|| {
                                let err_msg = "error finding tag, this should not happend";
                                error!("{}", err_msg);
                                err_msg
                            })?
                            .id
                    }
                };

                // Insert the title_id and tag_id to titles_tags
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
            }
        }
        /* #endregion */

        /* #region - encode pages to blurhashes */
        #[cfg(debug_assertions)]
        let start_timer = std::time::Instant::now();

        let page_blurhashes: Vec<BlurhashResult> =
            scan_extracted(&title_temp_dir, &self.image_formats)
                .await
                .par_iter()
                .filter_map(|page| self.blurhash.encode(&page.path, &page.ext))
                .collect();

        #[cfg(debug_assertions)]
        {
            let total_time = start_timer.elapsed().as_secs_f64();
            let time_per_page = total_time / page_blurhashes.len() as f64;
            debug!(
                "encode pages to blurhashes | total time: {:.2}s | time per page: {:.2}s",
                total_time, time_per_page
            );
        }
        /* #endregion */

        /* #region - clear old, push new page to DB */

        // Key: BHResult.file_name, Value: BHResult
        let page_blurhashes_map = page_blurhashes
            .iter()
            .map(|page| (page.file_name.clone(), page.clone()))
            .collect::<std::collections::HashMap<String, BlurhashResult>>();

        // Key: page_model.path, Value: page_model.id
        let page_models_map = Pages::find()
            .filter(pages::Column::TitleId.eq(&title_id))
            .all(&self.app_state.db)
            .await
            .map_err(|e| {
                error!("error search pages in DB: {}", e);
                e
            })?
            .into_iter()
            .map(|page| (page.path.clone(), page.id.clone()))
            .collect::<std::collections::HashMap<String, String>>();

        // For each `path` in `page_models_map`:
        // - If `path` is not in `page_blurhashes_map`, delete it from the database.
        for (path, page_id) in &page_models_map {
            if !page_blurhashes_map.contains_key(path) {
                let _ = Pages::delete_by_id(page_id)
                    .exec(&self.app_state.db)
                    .await
                    .map_err(|e| {
                        error!("error deleting old pages: {}", e);
                        e
                    })?;
            }
        }

        // For each `path` in `page_blurhashes_map`:
        // - If `path` is not in `page_models_map`, insert it into the database.
        // - If `path` is in `page_models_map`, update it in the database.
        for (path, blurhash) in &page_blurhashes_map {
            if !page_models_map.contains_key(path) {
                let _ = pages::ActiveModel {
                    id: Set(Uuid::new_v4().to_string()),
                    title_id: Set(title_id.clone()),
                    path: Set(blurhash.file_name.clone()),
                    blurhash: Set(blurhash.blurhash.clone()),
                    width: Set(blurhash.width),
                    height: Set(blurhash.height),
                    description: Set(title_metadata.get_page_desc(blurhash.file_name.clone())),
                }
                .insert(&self.app_state.db)
                .await
                .map_err(|e| {
                    error!("error inserting pages: {}", e);
                    e
                })?;
            } else {
                let page_model = Pages::find_by_id(page_models_map.get(path).unwrap())
                    .one(&self.app_state.db)
                    .await
                    .map_err(|e| {
                        error!("error finding page: {}", e);
                        e
                    })?
                    .ok_or_else(|| {
                        let err_msg = "error finding page, this should not happend";
                        error!("{}", err_msg);
                        err_msg
                    })?;
                if page_model.blurhash != blurhash.blurhash {
                    let mut active_page: pages::ActiveModel = page_model.into();
                    active_page.blurhash = Set(blurhash.blurhash.clone());
                    active_page.width = Set(blurhash.width);
                    active_page.height = Set(blurhash.height);
                    let _ = active_page.update(&self.app_state.db).await.map_err(|e| {
                        error!("error update page in DB: {}", e);
                        e
                    })?;
                }
            }
        }
        /* #endregion */

        /* #region - title thumbnail */
        let _ = Thumbnails::delete_many()
            .filter(thumbnails::Column::Id.eq(&title_id))
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
            blurhash_pages: &page_blurhashes,
            explicit_name: &title_metadata.thumbnail,
            implicit_names: &possible_thumbnail_name,
            formats: &self.image_formats,
            valid_pages: vec![],
        };
        if let Some(thumbnail) = thumbnail_finder.find() {
            info!("- thumbnail found: {}", thumbnail.file_name);

            // write BHResult (filename) -> <title>.toml
            title_metadata.set_thumbnail(thumbnail.file_name.clone());

            // write BHResult (everything) -> DB
            let _ = thumbnails::ActiveModel {
                id: Set(title_id),
                path: Set(thumbnail.file_name),
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
