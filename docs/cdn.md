# The CDN

## development info

speederboard's `dev` feature flag (enabled by default) **when run from the root directory of speederboard** will automagically start a CDN service task
which reads from and writes to `/tmp/speederboard-assets/`.

## prod info

The supported CDN configuration for speederboard is a Cloudflare R2 bucket. This bucket should be public,
with the cdn-upload-worker worker applied to it. This worker takes a SHA256 hash of some random string.
`openssl rand -hex 32` is sufficient to generate the secret, and `echo -n "input" | shasum -a 256` should generate
the shasum which can be passed in as a secret on your worker settings page.

## structure

The structure of the CDN is as follows:

```text
.
├── users/
│   └── :id/
│       ├── pfp.ext
│       ├── banner.ext
│       └── style.css
└── games/
    └── :id/
        ├── banner.ext
        └── coverart.ext
```

each .ext file is available as PNG with `png` as the extension. More file formats may be supported in the future.
