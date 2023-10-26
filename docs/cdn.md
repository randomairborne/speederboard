# The CDN

## development info

speederboard's `dev` feature flag (enabled by default) **when run from the root directory of speederboard** will automagically
start a CDN task for assets/public. You need to set up an S3-compatible (I use [minio](https://min.io)) server on your device,
with public bucket access and point `S3_ENDPOINT` at it. Most S3-compatible services do away with
`S3_REGION`, but setting it to `us-east-1` is usually sufficient.

`S3_ACCESS_KEY` should be set to your access key, and `S3_SECRET_KEY` to your secret key.

## prod info

The supported CDN configuration for speederboard is a Cloudflare R2 bucket. This bucket should be public,
with your cloudflare `R2_ACCOUNT_ID` in that config variable.

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
