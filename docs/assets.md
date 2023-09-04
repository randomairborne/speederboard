# assets in speederboard

speederboard makes use of many static assets. These are hosted on the CDN at `/public/`, and updated by GitHub Actions
from `assets/public/` to their production and staging locations.

Note that all sources for assets (Aseprite ASEs, Photoshop PSDs, etc) should be placed in the `assets/source` directory.
These are not synced to the CDN, and are purely for reference and future use.
