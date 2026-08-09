
<!-- Language tabs. GitHub has no README localisation, so this is a hand-written row of
     links; <kbd> is used because GitHub themes it for both light and dark mode. Add
     each language's own file. -->
<p align="right">
<kbd><b>English</b></kbd> &nbsp; <a href="README.ko.md"><kbd>한국어</kbd></a> &nbsp; <a href="README.ja.md"><kbd>日本語</kbd></a> &nbsp; <a href="README.zh.md"><kbd>中文</kbd></a>
</p>

<div align="center">

<!-- The banner carries the wordmark, so the title is not repeated as text below it. The
     tagline is, even though the banner says much the same thing: this file is the
     template the other languages follow, and their banner may not be in their language. -->
<img src="assets/banner-en.png" alt="Kaflow Search — fast desktop search for Kafka messages, locally indexed rather than scanned. Works with any Kafka, Avro and Protobuf, SASL / TLS / AWS MSK IAM." width="920" />

### A desktop search engine for your Kafka messages

**Find one message among millions — in seconds, on your own machine.**

`Local indexing` · `No extra infrastructure` · `No sign-up` · `Your data stays local`

**[⬇ Download](#download)** · **[Install guide](#install)** · [Issues](https://github.com/whsoul/kaflow-search/issues)

</div>

---

## What it does

|  |  |
|---|---|
| 🔎 **Search, not scan** | Keyword or field search across millions of indexed rows, in seconds |
| 🧩 **Compound conditions** | `AND` / `OR` / `NOT` over keys, headers and nested JSON fields |
| 🕒 **Narrow by time** | Exact time range, or drill down from a time-series chart |
| 🧬 **Decode payloads** | JSON · Avro · Protobuf — local schemas or Confluent Schema Registry |
| 📤 **Export results** | JSONL / CSV / TSV, optionally gzip or zstd |
| 🔐 **Connects where you are** | **AWS MSK (IAM)** · SASL/SCRAM · SSL/TLS · Confluent Schema Registry |
| 🔌 **Works offline** | Indexed data stays searchable when the cluster is unreachable |

⚡ **Fast** — seconds across millions of rows · searching never re-reads Kafka

🪶 **Light** — one desktop app · no server, connector or database · auto-cleanup within a limit you set

🧠 **Powerful** — nested JSON and header conditions · AND/OR/NOT · time-series drill-down · Avro/Protobuf · connects to AWS MSK and other managed clusters

🛡️ **Private** — installed on your machine, not a cloud service · messages never leave it · passwords never stored

🤝 **Easy on your cluster** — read-only, and it only reads while indexing or syncing. After that, repeated queries request nothing from the cluster

---

## Demo

<div align="center">
  <img src="assets/kaflow-search-demo.gif" alt="Kaflow Search demo" width="920" />
</div>

<!-- Video is a GitHub attachment, not a repo file. Plain links to attachment URLs
     render as an inline player; <video> tags are stripped. -->
<div align="center">
  <sub><a href="https://github.com/user-attachments/assets/aee0fc7e-2f7e-484c-bf23-de6455bf9af9">▶ Watch the full walkthrough (3 min)</a> — every step, at real speed</sub>
</div>

<!-- Screenshots still to come. Add these, then uncomment:
       assets/screenshots/{search,search-builder,message-detail,timeseries}.png
     Left commented so nothing renders broken.

<table>
  <tr>
    <td width="50%"><img src="assets/screenshots/search.png" alt="Search results" /><br/><sub><b>Search results</b></sub></td>
    <td width="50%"><img src="assets/screenshots/search-builder.png" alt="Condition builder" /><br/><sub><b>Condition builder</b></sub></td>
  </tr>
  <tr>
    <td width="50%"><img src="assets/screenshots/message-detail.png" alt="Message detail" /><br/><sub><b>Message detail</b></sub></td>
    <td width="50%"><img src="assets/screenshots/timeseries.png" alt="Time series" /><br/><sub><b>Time-series drill-down</b></sub></td>
  </tr>
</table>
-->

---

## Download

| OS | Architecture | File |
|---|---|---|
| macOS | Apple Silicon | `.dmg` |
| macOS | Intel | `.dmg` |
| Windows | x64 | ⏳ In progress — get notified when it lands ↓ |
| Linux | — | Planned based on demand — [let us know](https://github.com/whsoul/kaflow-search/issues) |

**[⬇ Get the latest release](https://github.com/whsoul/kaflow-search/releases/latest)** — download only from here.

**Requirements** — macOS 11 (Big Sur) or later · Apache Kafka 2.4 or later ([which environments](#supported-environments))

| | |
|---|---|
| **Install size** | about 100 MB |
| **Free space** | **On top of that**, allow **10 GB or more per cluster** for the local index (two clusters = 20 GB or more). The index limit is adjustable in settings |
| **Disk type** | **SSD recommended.** Indexing is write-heavy — an HDD works but will be noticeably slower |

Nothing else to install — no runtime, no database, no service.

> 🔔 **Want to know when a new version lands?** Use **[Watch ▾] → Custom → ☑ Releases** at the top
> of this repository and you'll be notified on every release — handy if you're waiting on a platform
> that's still in progress.

## Install

> ### ⚠️ Your OS will warn you. That is expected.
>
> Kaflow Search is **not yet registered as a verified developer** with Apple or Microsoft — that
> requires paid certificates this project hasn't purchased. So your OS flags it as coming from an
> unidentified developer. It does **not** mean anything harmful was found. **Allow it once** using
> the steps below and it runs normally from then on.
>
> *(If you want to, you can also [check that your download wasn't tampered with](#data-and-safety). Not required.)*
>
> <sub>If [support](#supporting-the-project) adds up, developer verification is the first thing it pays for.</sub>

<details open>
<summary><b>macOS</b> — System Settings → "Open Anyway"</summary>

1. Open the `.dmg` and drag **Kaflow Search** into your Applications folder.

   > 💡 **Don't launch it from inside the DMG.** Move it first, then run it from Applications —
   > otherwise you'll go through the approval twice. (Approval applies to one file, so approving
   > the copy inside the DMG doesn't carry over to the one you copied out.)

2. Launch it from Applications. When this appears, click **OK**:

   > `"Kaflow Search" can't be opened because Apple cannot check it for malicious software.`

3. Go to **System Settings → Privacy & Security**, and under **Security** click
   **[Open Anyway]**. Confirm once more in the dialog that follows.

   > `"Kaflow Search" was blocked from use because it is not from an identified developer.`

   *(The button only appears **right after** step 2 was blocked. If you don't see it, launch the app again.)*

**Some setups ask twice** — once after installing, once on first launch. Nothing is wrong; just
approve it the same way again. It won't ask after that.

<br/>

**To skip all of it**, run this once in Terminal instead, then open the app normally:

```bash
xattr -dr com.apple.quarantine "/Applications/Kaflow Search.app"
```

*(Once per installed version. Exact wording varies by macOS version.)*

</details>

## Getting started

| | Step | Note |
|---|---|---|
| **1** | Connect to a cluster | Passwords are **never saved** — entered per connection |
| **2** | Pick topics to index | Deserializer and cleanup policy are auto-suggested from a sample |
| **3** | Index | One pass over the topic. The app stays usable while it runs |
| **4** | Search | Keyword, or conditions across keys / headers / JSON fields |

<details>
<summary><b>Connecting to AWS MSK (IAM auth)</b></summary>

MSK isn't a separate menu — it lives **inside the SASL mechanism list**. On the connect screen:

1. **Security protocol** → `SASL_SSL` *(MSK requires TLS; `SASL_PLAINTEXT` will not connect)*
2. **SASL mechanism** → `AWS MSK IAM (IAM access key)`
3. If you have `~/.aws/credentials`, a **profile picker** appears — pick one and the access key and
   region are filled in for you. *(With a single profile it applies directly, no picker.)*
4. Otherwise, or for SSO-only profiles, enter them yourself: **AWS Access Key ID** /
   **AWS Secret Access Key** / **Region** (e.g. `ap-northeast-2` — where your MSK cluster lives)

Keys are **not stored** — they're used for that connection only, and re-entered or re-loaded from
the profile next time.

> If it fails with `Access denied`, the failure dialog offers a **minimum-privilege IAM policy JSON**
> you can copy. MSK returns the same message for a mistyped key, a wrong region, missing permissions
> and a permission boundary alike — so the app lists what to check instead of guessing the cause.

</details>

---

# Details

## Why a local index

| | |
|---|---|
| **The problem** | Kafka is a log, not a database — there's no way to look up "the message with this order ID" |
| **What Kaflow does** | Reads each topic once, builds a local index. Every later search hits the index, not Kafka |
| **Scope** | Searches what is **still in your topics** — it mirrors Kafka retention, it is not an archive |
| **Management tools** | No overlap, no conflict. Keep yours for operating the cluster; use Kaflow to *find things* |

## Data and safety

| | |
|---|---|
| **Messages, indexes, queries** | Processed on your machine. Never uploaded to any Kaflow server |
| **Passwords, tokens, secret keys** | **Never stored.** Held in memory for the session, re-entered on each connection |
| **Stored locally** (`~/.kaflow/`) | Non-secret metadata only — protocol, mechanism, username, certificate paths |
| **Writes to your cluster** | **None.** Read-only: `Metadata`, `ApiVersions`, `DescribeConfigs`, `DescribeTopicPartitions`, `Fetch`, `ListOffsets`. No `Produce`, no admin calls — it *cannot* alter topics, consumer groups or ACLs |
| **Read load on your cluster** | Only while indexing and syncing. Searching, exploring and exporting all run off the local index, so repeating them sends nothing to the cluster (opening a message's raw payload is the exception) |
| **Sent to Kaflow** | On startup only, for version support and announcements: random instance ID · hashed machine ID · app version · OS/arch · UI language. Nothing about your Kafka setup or messages |
| **Diagnostic reports** | Generated locally, sent only if you choose to |

Full detail, including retention: **[PRIVACY.md](legal/PRIVACY.md)**

<details>
<summary><b>Verifying your download wasn't tampered with</b> (optional)</summary>

> **This is not required to install or run the app.** Skip it and everything works fine.
> It's here for people who want the extra assurance.

Every release ships a **`SHA256SUMS`** file, and the release notes list the same **SHA-256**
values. Computing the fingerprint of your download and comparing it tells you the file wasn't
**corrupted in transit or swapped out along the way**.

```bash
# macOS — put SHA256SUMS in the same folder and check everything at once
shasum -a 256 -c SHA256SUMS
```

```powershell
# Windows — compute the value and compare it against the release notes by eye
Get-FileHash <file> -Algorithm SHA256
```

If a value doesn't match, don't use that file — delete it and download again.

⚠️ **Know the limit.** The file and its checksum live on the same page, so if the release page
itself were compromised, both would change together and this check wouldn't catch it. Guarding
against that is what developer verification (code signing) does — which this project doesn't have yet.

</details>

<details>
<summary><b>Before you connect</b></summary>

- Confirm you're authorized to read the cluster and topics
- Follow your organization's security and data-handling policy
- Local indexes and exports are **unencrypted** on disk — store accordingly
- Prefer a least-privilege Kafka account
- Strip credentials and message contents before sharing logs

</details>

## Supported environments

| | |
|---|---|
| **Security protocol** | `PLAINTEXT` · `SSL` · `SASL_PLAINTEXT` · `SASL_SSL` |
| **SASL mechanism** | `PLAIN` · `SCRAM-SHA-256` · `SCRAM-SHA-512` · `OAUTHBEARER` · **AWS MSK IAM** |
| **Certificates** | PEM · PKCS#12 (`.p12` / `.pfx`) |
| **Schema Registry** | Confluent (Avro, Protobuf, incl. schema references) |
| **Kafka versions** | Target **2.4 – 4.x** · **verified:** Kafka 2.4.x (local), AWS MSK with IAM |

Other versions and managed services are expected to work but aren't verified yet —
[tell us how it went](https://github.com/whsoul/kaflow-search/issues). Connection success also
depends on broker config, network policy and provider-specific auth, not the version alone.

<details>
<summary><b>What you can search</b></summary>

| Target | Example |
|---|---|
| Message key | `K.orderId` — plain value or nested JSON |
| Payload | `P.customer.email` — plain text or nested JSON |
| Header | `H.trace-id` |
| Arrays | `P.items[*].sku` — flattened to one path |

| Condition | |
|---|---|
| Match | exact · prefix · lexical range within a field |
| Boolean | `AND` / `OR` / `NOT`, with minimum-match count for `OR` |
| Filters | timestamp range · partition / offset range |
| Words | word-level matching on fields you mark as tokenized |

| Exploring | |
|---|---|
| Charts | time-series with drill-down into matching messages |
| Trees | offset tree · date tree · partition placement · latest messages |
| Detail | JSON highlighting, copy |
| Export | JSONL / CSV / TSV × gzip / zstd — runs off the local index, works offline |

</details>

<details>
<summary><b>Resource limits</b></summary>

Limits on disk, on scale, and on how indexes are kept and cleaned up are set so that
Kaflow stays well-behaved on your machine. The app shows the ones in force under
**Settings**, and enforces exactly what it shows.

Indexing time and disk use vary with message count, size, partitions, indexed fields and
disk speed. No fixed performance is guaranteed.

</details>

## Project status

**Version 0.1.0 — first public release.** Usable for daily work, but early, and the version number says so.

**Known limitations**

| | |
|---|---|
| One topic at a time | Multi-topic search is the next feature |
| Text-based ranges | Numeric/date range search (`amount 50–100`) not available yet |
| Manual updates | No auto-update — new versions are installed by hand |
| Unsigned | See [Install](#install) |

> ⚠️ No results means nothing matched **in your local index** — not that the message never existed
> in Kafka. It may predate your index, or fall outside the fields and range you indexed.

## Feedback

[Report a bug](https://github.com/whsoul/kaflow-search/issues/new) ·
[Request a feature](https://github.com/whsoul/kaflow-search/issues/new) ·
[All issues](https://github.com/whsoul/kaflow-search/issues)

For connection problems, include: OS and Kaflow version · Kafka service or distribution ·
approximate Kafka version · auth method · error message with secrets removed · whether other
Kafka clients connect.

> 🔒 **Never attach passwords, tokens, private keys, certificates or real business messages to a public issue.**

## Supporting the project

Built and maintained independently. Support is voluntary — not a purchase, subscription or
commitment to any feature or timeline. Free usage terms are the same either way.

**Where it would go first:** code-signing certificates for macOS and Windows, so the
[install warnings](#install) go away. After that — testing across more platforms and Kafka
versions, release infrastructure, and documentation.

## This repository

Official public repository — releases, documentation, issues, roadmap.

| | |
|---|---|
| **Source-available, not open source** | The production frontend and search engine are **not** here |
| **What is here** | Public API contracts · mock engine · desktop shell · a runnable demo build |
| **Why** | Transparency and evaluation. No open-source license applies unless a file says so — see [LICENSE](LICENSE) |
| **Contributions** | Code contributions aren't accepted. Issues and feedback very much are |

**Legal** — [LICENSE](LICENSE) (repository terms) ·
[EULA](legal/EULA.md) ([한국어](legal/EULA.ko.md)) ·
[Privacy](legal/PRIVACY.md) ([한국어](legal/PRIVACY.ko.md)) ·
[Third-party notices](legal/THIRD_PARTY_NOTICES.md)

Proprietary software, currently **free for personal use and internal business use** under the
EULA. Reselling, modifying, reverse-engineering, redistributing or bundling into another product
is restricted. Future features or services may carry different terms, announced before they apply.

The app's interface is available in **English · 한국어 · 日本語 · 中文**.

**Trademarks** — Apache Kafka and Kafka are trademarks of the Apache Software Foundation.
Kaflow Search is not affiliated with, endorsed by or sponsored by the ASF. Other names and marks
belong to their respective owners.

---

<div align="center">

**Kaflow Search** — a desktop search engine for your Kafka messages.

[Download](https://github.com/whsoul/kaflow-search/releases/latest) ·
[Issues](https://github.com/whsoul/kaflow-search/issues) ·
[Privacy](legal/PRIVACY.md)

Copyright © 2026 [PLACEHOLDER: LEGAL_NAME], operating as "Kaflow Search".

</div>
