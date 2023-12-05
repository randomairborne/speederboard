set CONSTRAINTS ALL DEFERRED;
INSERT INTO games (id, name, slug, url, default_category, description, banner, cover_art, flags)
VALUES
    (1, 'Test game', 'test', 'https://example.com', 1, 'Test game for speederboard', false, false, 0);
INSERT INTO categories (id, game, name, description, rules, scoreboard, flags)
VALUES
    (1, 1, 'test category', 'test category', '(test)', false, 0);
