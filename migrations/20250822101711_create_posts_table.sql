-- Add migration script here
CREATE TABLE IF NOT EXISTS posts
(
    id         INTEGER PRIMARY KEY AUTOINCREMENT   NOT NULL,
    title      TEXT                                NOT NULL,
    author_id  INTEGER                             NOT NULL,
    content    TEXT                                NOT NULL,
    tags       TEXT                                NOT NULL,
    copyright  TEXT                                NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,

    FOREIGN KEY (author_id) REFERENCES users (id)
);

-- 为 posts 表添加软删除字段
ALTER TABLE posts
    ADD COLUMN deleted_at TIMESTAMP;