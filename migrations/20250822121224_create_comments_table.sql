-- Add migration script here
CREATE TABLE IF NOT EXISTS comments
(
    comment_id INTEGER PRIMARY KEY                 NOT NULL,
    post_id    INTEGER                             NOT NULL,
    author_id  INTEGER                             NOT NULL,
    content    TEXT                                NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
    FOREIGN KEY (post_id) REFERENCES posts (id) ON DELETE CASCADE,
    FOREIGN KEY (author_id) REFERENCES users (id)
);

-- 为 comments 表添加软删除字段
ALTER TABLE comments
    ADD COLUMN deleted_at TIMESTAMP;