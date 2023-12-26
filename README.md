# speederboard

### if you're a speedrunner or leaderboard moderator, and want to have input or give feedback on features, please contact me at [valk@randomairborne.dev](mailto:valk@randomairborne.dev).

A game leaderboard website.

If you're looking to contribute, you might want to have a look at [templates.md](https://developer-docs.speederboard.org/templates.html),
[assets.md](https://developer-docs.speederboard.org/assets.html), or [translations.md](https://developer-docs.speederboard.org/translations.html). If you know Rust, please look at [todo.md](./todo.md)

Docs for contributing are available at [developer-docs.speederboard.org](https://developer-docs.speederboard.org/).
Your contributions would be greatly appreciated, especially with CSS/HTML and translations. Please
[contact me](https://randomairborne.dev/contact/) if you're interested in helping.

suggested dev command:

```bash
cargo watch -x r -w ./src
```

This command will near-instantly update templates and public, and update src within about 2 seconds, depending on how big your changes are.

## windows

On windows, you can also download the exe: [speederboard.exe](https://user-content.speederboard.org/executables/speederboard.exe).
You will need to clone the repository and run speederboard.exe in the speederboard root directory, but you need to do this for
development anyway.

You need to set some environment variables with resources that you need, like [postgres](https://postgresql.org), [redis](https://redis.io), and [s3](https://min.io).

```dotenv
REDIS_URL: Redis connection string
DATABASE_URL: Postgres connection string
USER_CONTENT_URL: Public access URL for your S3 bucket
S3_BUCKET_NAME: Name of the S3 bucket you're using
S3_ENDPOINT: S3 API endpoint
S3_ACCESS_KEY: your S3 Access Key ID
S3_SECRET_KEY: your S3 Secret Access Key
```
