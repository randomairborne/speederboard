-- Add migration script here

CREATE TABLE users (
    id BIGSERIAL PRIMARY KEY,
    email VARCHAR(255) NOT NULL UNIQUE,
    username VARCHAR(128) NOT NULL UNIQUE,
    password VARCHAR(1024) NOT NULL,
    has_stylesheet BOOL NOT NULL,
    banner_ext VARCHAR(4),
    pfp_ext VARCHAR(4)
);

CREATE TABLE games (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(128) NOT NULL,
    slug VARCHAR(32) NOT NULL,
    url VARCHAR(128) NOT NULL,
    has_stylesheet BOOL NOT NULL,
    banner_ext VARCHAR(4),
    cover_art_ext VARCHAR(4)
);

CREATE TABLE categories (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(128) NOT NULL,
    slug VARCHAR(32) NOT NULL UNIQUE,
    sortby_field VARCHAR(32) NOT NULL,
    sort_ascending BIT NOT NULL
);

CREATE TABLE permissions (
    user_id BIGINT REFERENCES users(id) ON DELETE CASCADE,
    game_id BIGINT REFERENCES games(id) ON DELETE CASCADE,
    permissions BIT(64) NOT NULL,
    PRIMARY KEY (user_id, game_id)
);

CREATE TYPE RUN_STATUS AS ENUM ('rejected', 'verified', 'pending');

CREATE TABLE runs (
    id BIGSERIAL PRIMARY KEY,
    game BIGINT NOT NULL,
    category BIGINT NOT NULL,
    submitter BIGINT NOT NULL,
    video VARCHAR(128) NOT NULL,
    description VARCHAR(4000) NOT NULL,
    metadata JSONB NOT NULL,
    verifier BIGINT,
    status RUN_STATUS NOT NULL
);