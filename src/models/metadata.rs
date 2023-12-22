use std::path::PathBuf;
use toml_edit::Document;
use tracing::{info, warn};

async fn try_read_file(path: &PathBuf) -> Option<Document> {
    if !path.exists() {
        info!("file does not exist: {}\n", path.to_string_lossy());
        return None;
    };

    let toml_file = match tokio::fs::read_to_string(path).await {
        Ok(toml_file) => toml_file,
        Err(e) => {
            warn!("error reading toml file: {}\n", e);
            return None;
        }
    };

    match toml_file.parse::<Document>() {
        Ok(doc) => Some(doc),
        Err(e) => {
            warn!("error parsing toml file: {}\n", e);
            None
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct TitleMetadata {
    pub title: Option<String>,
    pub description: Option<String>,
    pub thumbnail: Option<String>,
    pub author: Option<String>,
    pub title_image: Option<String>,
    pub release_date: Option<String>,
    pub tags: Option<Vec<String>>,
    /// Per-page description
    /// 1st element is the page number
    /// 2nd element is the description
    pub descriptions: Option<Vec<(String, String)>>,
    doc: Document,
}

impl TitleMetadata {
    fn parse_string(&self, s: &str) -> Option<String> {
        if self.doc[s].is_none() {
            return None;
        }
        self.doc[s].as_str().map(|s| s.to_string())
    }

    fn parse_array(&self, s: &str) -> Option<Vec<String>> {
        if self.doc[s].is_none() {
            return None;
        }
        self.doc[s].as_array().map(|a| {
            a.iter()
                .map(|v| v.as_str().unwrap_or_default().to_string())
                .collect()
        })
    }

    fn parse_table(&self, s: &str) -> Option<Vec<(String, String)>> {
        if self.doc[s].is_none() {
            return None;
        }
        let mut result = Vec::new();
        if let Some((k, v)) = self.doc["descriptions"].as_table()?.iter().next() {
            let page_number = k.to_string();
            let description = v.as_str().unwrap_or_default().to_string();
            result.push((page_number, description));
        }
        Some(result)
    }

    pub async fn read_from_file(&mut self, path: &PathBuf) {
        match try_read_file(path).await {
            Some(doc) => self.doc = doc,
            None => return,
        }

        self.title = self.parse_string("title");
        self.title_image = self.parse_string("title_image");
        self.description = self.parse_string("description");
        self.author = self.parse_string("author");
        self.tags = self.parse_array("tags");
        self.thumbnail = self.parse_string("thumbnail");
        self.release_date = self.parse_string("release_date");
        self.descriptions = self.parse_table("descriptions");
    }

    /// Return the description that matches the page filename
    pub fn get_page_desc(&self, path: String) -> Option<String> {
        let path = PathBuf::from(path);
        let no_ext = path
            .file_stem()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default();
        let with_ext = path
            .file_name()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default();
        // rewrite using the Rust way of iterating
        self.descriptions
            .iter()
            .flatten()
            .find(|(page_number, _)| page_number == no_ext || page_number == with_ext)
            .map(|(_, description)| description.clone())
    }
}

#[derive(Debug, Clone, Default)]
pub struct CategoryMetadata {
    pub title: Option<String>,
    pub description: Option<String>,
    pub thumbnail: Option<String>,
    doc: Document,
}

impl CategoryMetadata {
    fn parse_string(&self, s: &str) -> Option<String> {
        if self.doc[s].is_none() {
            return None;
        }
        self.doc[s].as_str().map(|s| s.to_string())
    }

    pub async fn read_from_file(&mut self, path: &PathBuf) -> Option<()> {
        match try_read_file(path).await {
            Some(doc) => self.doc = doc,
            None => return None,
        }

        self.title = self.parse_string("title");
        self.description = self.parse_string("description");
        self.thumbnail = self.parse_string("thumbnail");
        Some(())
    }
}
