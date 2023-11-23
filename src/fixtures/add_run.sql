INSERT INTO runs
    (id, game, category, submitter, video, description, score, time, verifier, status, created_at, verified_at, edited_at, flags)
VALUES
    (1, 1, 1, 1, 'https://www.youtube.com/watch?v=vOLivyykLqk', 'test run', 0, 0, NULL, 0, cast(to_timestamp(0) as timestamp), NULL, NULL, 0);