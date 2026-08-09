# assets

Images used by the README, plus artwork kept for other channels.

All of these are 1774×887 (2:1) unless noted.

| File | What it is | Where it is used |
|---|---|---|
| `banner-en.png` | Brand banner — logo, headline, one compatibility line | README header (both languages) |
| `banner-en.jpg` | Same banner, 501 KB | Social preview upload (GitHub wants ≤ 1 MB, 2:1) |
| `promo-card-en.png` | Dense promo card — four feature tiles, a product screenshot, and a four-step strip | Not in the README. Kept for posts and listings that get a full-width image |
| `promo-card-ko.png` | Korean promo card | **Not publishable as is** — see below |
| `kaflow-search-demo.gif` | Demo loop, 37.7 s, 1380 px | README demo section |
| `kaflow-icon.png` | App icon | Not in the README |

The full walkthrough video is a GitHub attachment rather than a file here, so it does not
count against repository size.

## Choosing between the banner and the promo card

The banner survives being shown small; the promo card does not. A social preview is
rendered around 500 px wide in a feed, where the promo card's tile text and screenshot
rows become unreadable. Use the promo card only where it gets full width.

## promo-card-ko.png

The product screenshot in the Korean card was redrawn rather than captured, and the
redraw invented interface that does not exist — a `SAVED SEARCHES` panel — alongside
garbled labels (`QUICK SEARCH SICH`, `Café`, `Topict`, `ichat-messages`) and timestamps
from the wrong year. Publishing it would advertise a feature we do not ship. It needs a
real screenshot before it can be used.

The English card's screenshot is a real capture and has none of these problems.

## Licensing

These images are part of the Kaflow Search brand and are covered by the repository
[LICENSE](../LICENSE) — they are not licensed for reuse.
