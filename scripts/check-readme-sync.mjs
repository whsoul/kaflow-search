#!/usr/bin/env node
// Keeps the translated READMEs from silently drifting away from the English one.
//
// Nobody on this project reads every language, so a section added to README.md and
// forgotten everywhere else would go unnoticed until a reader hit it. This compares the
// shape of the files rather than their words: heading levels, table sizes, collapsible
// blocks, external links. Those have to match even though the prose does not.
//
// It also resolves every in-page link against the headings of its own file. GitHub only
// generates anchors for markdown headings — a link pointing at <summary> text looks
// right and does nothing, which is how one got shipped.
//
//   node scripts/check-readme-sync.mjs

import { readFileSync } from "node:fs";

const SOURCE = "README.md";
const TRANSLATIONS = ["README.ko.md", "README.ja.md", "README.zh.md"];

// GitHub lowercases the heading, drops punctuation, and turns spaces into hyphens.
// Letters and digits of any script survive, which is what makes non-English anchors work.
function slug(heading) {
  return heading
    .replace(/<[^>]*>/g, "")
    .replace(/[`*_]/g, "")
    .trim()
    .toLowerCase()
    .replace(/[^\p{L}\p{N} -]/gu, "")
    .trim()
    .replace(/ +/g, "-");
}

function parse(path) {
  const raw = readFileSync(path, "utf8").replace(/<!--[\s\S]*?-->/g, "");
  const headings = [];
  const tables = [];
  let run = 0;
  let inFence = false;

  for (const line of raw.split("\n")) {
    if (/^\s*```/.test(line)) {
      inFence = !inFence;
      continue;
    }
    if (inFence) continue;

    // Headings inside blockquote callouts still count — GitHub anchors them too.
    const text = line.replace(/^\s*>\s?/, "");
    const heading = /^(#{1,6}) +(.*)$/.exec(text);
    if (heading) headings.push({ level: heading[1].length, text: heading[2] });

    if (/^\s*\|/.test(text)) run += 1;
    else if (run) {
      tables.push(run);
      run = 0;
    }
  }
  if (run) tables.push(run);

  return {
    path,
    headings,
    tables,
    details: (raw.match(/<details/g) || []).length,
    // Links have to match; images only have to be equally numerous. An image is allowed
    // to differ per language — a label badge carries translated text, and a banner may
    // one day be drawn in each language — whereas a link goes to the same place for
    // everyone, so a missing or stray one is a mistake.
    links: [...raw.matchAll(/(^|[^!])\[[^\]]*\]\((https?:\/\/[^)\s]+)\)/g)].map((m) => m[2]).sort(),
    images: (raw.match(/!\[[^\]]*\]\(/g) || []).length,
    anchors: [...raw.matchAll(/\]\((#[^)\s]+)\)/g)].map((m) => m[1]),
  };
}

const problems = [];
const fail = (file, msg) => problems.push(`${file}: ${msg}`);

const files = [SOURCE, ...TRANSLATIONS].map(parse);

// Every in-page link has to land on a heading in the same file.
for (const f of files) {
  const targets = new Set(f.headings.map((h) => slug(h.text)));
  for (const a of f.anchors) {
    const want = decodeURIComponent(a.slice(1)).toLowerCase();
    if (!targets.has(want)) fail(f.path, `link ${a} matches no heading in this file`);
  }
}

// The translations have to keep the same shape as the English original.
const [src, ...rest] = files;
for (const f of rest) {
  const levels = (x) => x.headings.map((h) => h.level).join(",");
  if (levels(f) !== levels(src)) {
    fail(f.path, `has ${f.headings.length} headings, ${SOURCE} has ${src.headings.length}` +
      ` — or they are at different levels. A section was added or dropped.`);
  }
  if (f.tables.join(",") !== src.tables.join(",")) {
    fail(f.path, `table rows are [${f.tables}], ${SOURCE} has [${src.tables}]` +
      ` — a row was added or dropped.`);
  }
  if (f.details !== src.details) {
    fail(f.path, `has ${f.details} collapsible blocks, ${SOURCE} has ${src.details}.`);
  }
  if (f.images !== src.images) {
    fail(f.path, `has ${f.images} images, ${SOURCE} has ${src.images}.`);
  }
  // Counted, not merely looked up. The same URL appears several times over — the issue
  // tracker is linked from the header, from a table row and from the feedback section —
  // so asking only whether it appears at all would let one of them go missing unnoticed.
  const tally = (urls) => urls.reduce((m, u) => m.set(u, (m.get(u) || 0) + 1), new Map());
  const want = tally(src.links);
  const got = tally(f.links);
  for (const url of new Set([...want.keys(), ...got.keys()])) {
    const n = want.get(url) || 0;
    const m = got.get(url) || 0;
    if (n !== m) fail(f.path, `links ${url} ${m} time(s), ${SOURCE} links it ${n}.`);
  }
}

if (problems.length) {
  console.error("README translations are out of sync:\n");
  for (const p of problems) console.error(`  ${p}`);
  console.error(`\n${problems.length} problem(s). Update every language together.`);
  process.exit(1);
}

console.log(`READMEs are in sync (${files.length} languages, ${src.headings.length} sections).`);
