-- Add migration script here
CREATE TABLE IF NOT EXISTS users
(
    id              TEXT PRIMARY KEY NOT NULL,
    username        TEXT             NOT NULL,
    email           TEXT             NOT NULL,
    profile_picture TEXT                     ,
    created_at      DATETIME         NOT NULL,
    updated_at      DATETIME         NOT NULL,
    password        TEXT             NOT NULL
);
