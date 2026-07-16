# Third-Party Notices

Kaflow Search includes third-party open-source components. Each component is
governed by its own license terms, which are not limited by the Kaflow Search
EULA or by the repository's licensing structure.

> **STATUS: placeholder — the generated inventory must be produced before the
> first public release.** (See "Generation" below. Tracked as a named task in
> the project status docs.)

## Generation

The inventory is to be generated at release-build time, not maintained by hand:

- **Rust dependencies** — `cargo about generate` (or `cargo license`) over the
  workspace, release feature set (`engine-impl`, no `debug-api`).
- **npm/pnpm dependencies** — `pnpm licenses list --prod` (or
  `license-checker`) over the production FE bundle inputs.
- **Tauri and plugins** — included in the cargo pass.
- **Fonts, icons, images from external sources** — manual entries appended
  below the generated section.

Each entry should include: component name, version, copyright holder, license,
and the license text (or a pointer to it), plus the project homepage or
repository where available.

Where an upstream component's license requires preservation of a NOTICE file
or attribution text in distributions (e.g., Apache-2.0 dependencies), that
content must be reproduced in this file (or shipped alongside it).

## Distribution

This file ships with:

- official installer packages (alongside `EULA.md`), and
- the application's Settings/About → "Open Source Licenses" screen.
