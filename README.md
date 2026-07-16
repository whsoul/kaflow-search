# Kaflow Search

Kaflow Search is a desktop application for searching and exploring Apache
Kafka topic messages, with fast local indexing (RocksDB) and multi-condition
search — built for day-to-day debugging and incident digging.

**Kaflow Search is free to use** for personal use and for internal business
use within your organization. See [EULA.md](EULA.md).

## ⚠️ Source-available, not open source

This repository is **source-available**: it exists for transparency,
documentation, releases, issue tracking, and community feedback. The
application's production source code (frontend and engine) is **not** in
this repository, and no open-source license applies here unless a file
explicitly states otherwise. You may build and evaluate what is in this
repository; see [LICENSE](LICENSE) for exactly what is and isn't permitted.

Selected components (public API contracts, mock implementations) may be
released under an open-source license in the future.

## Downloads

Official installers are published on the [Releases] page and at
[Official Website]. Each release includes SHA-256 checksums. Download only
from official sources.

## Your data stays local

- Kafka messages, topic names, cluster addresses, credentials, local indexes,
  search queries: **processed locally, never uploaded to Kaflow servers.**
- Passwords are not persisted to disk.
- The app contacts the Kaflow app-check service only for version/compatibility
  checks, sending a small fixed set of technical fields (random instance ID,
  hashed machine ID, app version, OS, locale). Details, including server log
  retention: [PRIVACY.md](PRIVACY.md).
- Diagnostic reports are generated locally and shared only if you choose to.

## What this repository is for

- 📦 **Releases** — official installers + checksums
- 🐛 **Issues** — bug reports and feature requests
- 📖 **Docs** — usage and integration documentation
- 🗺️ **Roadmap / notices**

Code contributions are not currently accepted (see LICENSE §7); issues and
feedback are very welcome.

## Legal

- [LICENSE](LICENSE) — repository terms (source-available)
- [EULA.md](EULA.md) ([한국어](EULA.ko.md)) — application license
- [PRIVACY.md](PRIVACY.md) ([한국어](PRIVACY.ko.md)) — privacy notice
- [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) — third-party licenses

Kaflow Search, its name, logo, and icons are proprietary brand assets.

---

Copyright © 2026 [Legal Name], operating under the name "Kaflow Search".
All rights reserved.
