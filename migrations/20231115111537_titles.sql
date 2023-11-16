CREATE TABLE IF NOT EXISTS titles
(
    id           TEXT PRIMARY KEY NOT NULL,
    title        TEXT             NOT NULL,
    category_id  TEXT                     ,
    author       TEXT                     ,
    description  TEXT                     ,
    release_date DATETIME                 ,
    is_colored   BOOLEAN                  ,
    is_completed BOOLEAN                  ,
    thumbnail    TEXT                     ,
    FOREIGN KEY(category_id) REFERENCES categories(id) ON DELETE CASCADE
);