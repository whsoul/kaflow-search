# Kaflow Search website

Static promotional site for Kaflow Search. It has no build step or runtime dependencies.

## Preview locally

```sh
python3 -m http.server 8080 --directory website
```

Then open <http://localhost:8080>.

## Deploy to Cloudflare

Use `website` as the static asset directory when creating the Cloudflare project.

## Before production launch

Once the final domain is known, add its absolute URL as the canonical URL, Open Graph URL,
and `sitemap.xml` location. Add a purpose-made social preview image as `og:image` too.
