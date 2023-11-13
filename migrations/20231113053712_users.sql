-- Add migration script here
CREATE TABLE IF NOT EXISTS users
(
    id              TEXT PRIMARY KEY NOT NULL,
    email           TEXT             NOT NULL,
    profile_picture TEXT             NOT NULL,
    created_at      DATETIME                 ,
    updated_at      DATETIME                 ,
);
