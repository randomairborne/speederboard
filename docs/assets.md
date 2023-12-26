# assets in speederboard

speederboard makes use of many static assets. These are hosted on the static CDN at `/`, and updated by GitHub Actions
from `assets/public/` to their production and staging locations.
These files can be edited freely, such as the style.css file.

Note that all sources for assets (Aseprite ASEs, Photoshop PSDs, etc) should be placed in the `assets/source` directory.
These are not synced to the CDN, and are purely for reference and future use.

## cache-busting

We use query parameters for cache-busting. These are precalculated on every launch
using blake3, but shouldn't be of concern to frontend devs.
