-- Add migration script here
CREATE TABLE IF NOT EXISTS users
(
    id            INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    username      TEXT UNIQUE                       NOT NULL,
    password_hash TEXT                              NOT NULL,
    role          TEXT                              NOT NULL DEFAULT 'user'
);