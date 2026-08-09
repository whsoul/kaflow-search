#!/usr/bin/env node
// Checks the committed frontend bundle.
//
// dist/ is not built here — the frontend sources live in the authoring repository, and
// this directory is the compiled result copied in. So there is nothing to lint or type
// check. What can go wrong is what a *published artifact* gets wrong: shipping the debug
// map alongside it, shipping a build pointed at the wrong backend, or shipping paths from
// the machine that built it. None of those show up as a failed build anywhere.
//
//   node scripts/check-dist.mjs

import { readFileSync, readdirSync, statSync } from "node:fs";
import { join, relative } from "node:path";

const DIST = "dist";
const problems = [];

function walk(dir) {
  const out = [];
  for (const entry of readdirSync(dir)) {
    const p = join(dir, entry);
    if (statSync(p).isDirectory()) out.push(...walk(p));
    else out.push(p);
  }
  return out;
}

let files;
try {
  files = walk(DIST);
} catch {
  console.error(`${DIST}/ is missing. The demo build embeds this path at compile time.`);
  process.exit(1);
}

// A .map file undoes the one thing this directory is for: it restores the original
// identifiers and module structure of a frontend that is deliberately not published.
for (const f of files.filter((f) => f.endsWith(".map"))) {
  problems.push(`${f} — sourcemap. Check build.sourcemap in the frontend's vite config.`);
}

// tauri::generate_context! resolves frontendDist at compile time, so an empty or missing
// entry point fails the Rust build with an error that says nothing about the bundle.
const indexPath = join(DIST, "index.html");
let index = "";
try {
  index = readFileSync(indexPath, "utf8");
  if (!index.trim()) problems.push(`${indexPath} is empty.`);
} catch {
  problems.push(`${indexPath} is missing.`);
}

// A bundle that references assets it did not ship renders as a blank window, and only
// at runtime.
const present = new Set(files.map((f) => relative(DIST, f).replace(/\\/g, "/")));
for (const m of index.matchAll(/(?:src|href)="\/([^"]+)"/g)) {
  if (!present.has(m[1])) problems.push(`${indexPath} references ${m[1]}, which is not in ${DIST}/.`);
}

// Everything below reads the bundled text. Binary assets have nothing to say.
const TEXT = /\.(js|css|html|json|svg|txt|map)$/;

// A development build looks identical from the outside but talks to the wrong backend.
// Users would be pointed at an environment that is not meant to serve them.
//
// The machine that produced the bundle should not be legible from it — an absolute path
// carries the developer's account name and directory layout.
const FORBIDDEN = [
  [/kaflow-api-dev\b/, "a development API host — this looks like a development build"],
  [/\/Users\/[A-Za-z0-9._-]+/, "an absolute macOS path from the build machine"],
  [/\/home\/[A-Za-z0-9._-]+/, "an absolute Linux path from the build machine"],
];

for (const f of files.filter((f) => TEXT.test(f))) {
  const text = readFileSync(f, "utf8");
  for (const [pattern, why] of FORBIDDEN) {
    const hit = pattern.exec(text);
    if (hit) problems.push(`${f} contains ${why}: ${JSON.stringify(hit[0].slice(0, 60))}`);
  }
}

if (problems.length) {
  console.error(`${DIST}/ is not fit to publish:\n`);
  for (const p of problems) console.error(`  ${p}`);
  process.exit(1);
}

console.log(`${DIST}/ is clean (${files.length} files).`);
