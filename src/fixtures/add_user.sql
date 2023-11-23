INSERT INTO users
(id, email, username, password, biography, admin, stylesheet, banner, pfp, flags, created_at, language)
VALUES
    (1, 'test@example.com', 'test', '$argon2id$v=19$m=4096,t=3,p=1$c2FsdG5wZXBwZXI$HSWAIFe7el+sIlef8Un8420qYOzYhouxfvHUbHG/q3s', '', false, false, false, false, 0, cast(to_timestamp(0) as timestamp), NULL);