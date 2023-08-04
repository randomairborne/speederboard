-- Add migration script here

CREATE TABLE users (
    id BIGINT PRIMARY KEY,
    email VARCHAR(255) NOT NULL,
    username VARCHAR(128) NOT NULL,
    password VARCHAR(1024) NOT NULL
);

CREATE TABLE games (
    id BIGINT PRIMARY KEY,
    name VARCHAR(128) NOT NULL,
    slug VARCHAR(32) NOT NULL,
    url VARCHAR(128) NOT NULL
);

CREATE TABLE categories (
    id BIGINT PRIMARY KEY,
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
    id BIGINT PRIMARY KEY,
    game BIGINT NOT NULL,
    category BIGINT NOT NULL,
    submitter BIGINT NOT NULL,
    video VARCHAR(128) NOT NULL,
    description VARCHAR(4000) NOT NULL,
    metadata JSONB NOT NULL,
    verifier BIGINT,
    status RUN_STATUS NOT NULL
);