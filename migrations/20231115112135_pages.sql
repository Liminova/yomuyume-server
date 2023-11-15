CREATE TABLE IF NOT EXISTS pages
(
    id       TEXT PRIMARY KEY NOT NULL,
    title_id TEXT             NOT NULL,
    path     TEXT             NOT NULL,
    hash     TEXT             NOT NULL
    width    INTEGER          NOT NULL,
    height   INTEGER          NOT NULL,
    FOREIGN KEY(title_id) REFERENCES titles(id) ON DELETE CASCADE
);