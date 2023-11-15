-- Add migration script here

CREATE TABLE users (
    id BIGSERIAL PRIMARY KEY,
    email VARCHAR(255) NOT NULL UNIQUE,
    username VARCHAR(128) NOT NULL UNIQUE,
    password VARCHAR(1024) NOT NULL,
    biography VARCHAR(4000) NOT NULL,
    admin BOOL NOT NULL DEFAULT false,
    stylesheet BOOL NOT NULL,
    banner BOOL NOT NULL,
    pfp BOOL NOT NULL,
    flags BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMP NOT NULL,
    language VARCHAR(5)
);

CREATE INDEX users_name_index ON users USING HASH (username);
CREATE UNIQUE INDEX case_insensitive_name_index ON users (lower(username));
CREATE INDEX users_email_index ON users USING HASH (email);

CREATE TABLE games (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(128) NOT NULL,
    slug VARCHAR(32) NOT NULL UNIQUE,
    url VARCHAR(128) NOT NULL,
    default_category BIGINT NOT NULL,
    description VARCHAR(4000) NOT NULL,
    banner BOOL NOT NULL,
    cover_art BOOL NOT NULL,
    flags BIGINT NOT NULL DEFAULT 0
);

CREATE INDEX games_slug_index ON games USING HASH (slug);

CREATE TABLE categories (
    id BIGSERIAL PRIMARY KEY,
    game BIGINT NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    name VARCHAR(128) NOT NULL,
    description VARCHAR(4000) NOT NULL,
    rules TEXT NOT NULL,
    scoreboard BOOL NOT NULL DEFAULT false,
    flags BIGINT NOT NULL DEFAULT 0
);

CREATE INDEX categories_game_index ON categories USING HASH (game);

CREATE TABLE permissions (
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    game_id BIGINT NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    permissions BIGINT NOT NULL,
    PRIMARY KEY (game_id, user_id)
);

CREATE INDEX permissions_quick_lookup_index ON permissions USING HASH (user_id);

CREATE TABLE runs (
    id BIGSERIAL PRIMARY KEY,
    game BIGINT NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    category BIGINT NOT NULL REFERENCES categories(id) ON DELETE CASCADE,
    submitter BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    video VARCHAR(256) NOT NULL,
    description VARCHAR(4000) NOT NULL,
    score BIGINT NOT NULL,
    time BIGINT NOT NULL,
    verifier BIGINT,
    status SMALLINT NOT NULL,
    created_at TIMESTAMP NOT NULL,
    verified_at TIMESTAMP,
    flags BIGINT NOT NULL DEFAULT 0
);

CREATE INDEX runs_category_index ON runs USING HASH (category);
CREATE INDEX runs_submitter_index ON runs USING HASH (submitter);
CREATE INDEX runs_score_index ON runs (score);
CREATE INDEX runs_time_index ON runs (time);

CREATE TABLE forum_entries (
    id BIGSERIAL PRIMARY KEY,
    title VARCHAR(256),
    parent BIGINT REFERENCES forum_entries(id) ON DELETE CASCADE,
    game BIGINT REFERENCES games(id) ON DELETE CASCADE,
    author BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMP NOT NULL,
    content VARCHAR(4000) NOT NULL,
    flags BIGINT NOT NULL DEFAULT 0,
    CONSTRAINT root_forum_entries_have_titles CHECK
    (
        (title IS NOT NULL AND parent IS NULL)
        OR
        (title IS NULL AND parent IS NOT NULL)
    )
);

CREATE INDEX forum_comment_parent_index ON forum_entries USING HASH (parent);
CREATE INDEX forum_post_game_index ON forum_entries USING HASH (game);
CREATE INDEX forum_by_user_lookup_index ON forum_entries USING HASH (author);
CREATE INDEX forum_time_sort_index ON forum_entries (created_at);
