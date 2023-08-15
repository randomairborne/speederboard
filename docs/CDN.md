# The CDN

## general info

the supported CDN configuration for speederboard is a Cloudflare R2 bucket. This bucket should be public,
with the cdn-upload-worker worker applied to it. This worker takes a SHA256 hash of some random string.
`openssl rand -hex 32` is sufficient to generate the secret, and `echo -n "input" | shasum -a 256` should generate
the shasum which can be passed in as a secret on your worker settings page.

## structure

The structure of the CDN is as follows:

```
.
├── static
└── customfiles/
    ├── users/
    │   └── :id/
    │       ├── pfp.png
    │       ├── banner.png
    │       └── style.css
    └── games/
        └── :id/
            ├── banner.png
            ├── style.css
            └── coverart.png
```
