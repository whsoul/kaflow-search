# Kaflow Search website

Static promotional site for Kaflow Search. It has no build step or runtime dependencies.

## Preview locally

```sh
python3 -m http.server 8080 --directory website
```

Then open <http://localhost:8080>.

## Deploy to Cloudflare

Use `website` as the static asset directory when creating the Cloudflare project.

## Production metadata

The production sitemap is available at `/sitemap.xml` and is advertised in `robots.txt`.
The canonical and Open Graph URLs are set in `index.html`. If the production domain
changes, update all three files together.

Add a purpose-made social preview image as `og:image` when one is available.
