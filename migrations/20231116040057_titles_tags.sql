CREATE TABLE IF NOT EXISTS titles_tags
(
    id       INTEGER PRIMARY KEY NOT NULL AUTOINCREMENT,
    title_id TEXT                NOT NULL              ,
    tag_id   INTEGER             NOT NULL              ,
    FOREIGN KEY(title_id) REFERENCES titles(id) ON DELETE CASCADE,
    FOREIGN KEY(tag_id)   REFERENCES tags(id)   ON DELETE CASCADE,
);