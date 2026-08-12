# Security Policy

## Reporting a vulnerability

Please report security issues **privately**, not as a public issue.

Use **[Report a vulnerability](https://github.com/whsoul/kaflow-search/security/advisories/new)**
on this repository's Security tab. It opens a private thread visible only to the maintainer.
If that isn't available to you, email **security@whsoul-tools.com** instead.

Please include what you were doing, what happened, and enough detail to reproduce it. If you
believe the issue is being exploited, say so in the first line.

This is a small project maintained by one person. You should get a first reply within a week.
If you hear nothing after that, assume the message did not arrive and follow up by email.

## Supported versions

Only the latest release receives fixes. Kaflow Search has no auto-update, so applying a fix
means downloading the new version from the
[releases page](https://github.com/whsoul/kaflow-search/releases).

## What is in scope

| | |
|---|---|
| **In scope** | The desktop application, the published crates in this repository, and the online services it contacts |
| **Out of scope** | Your Kafka cluster, Schema Registry, or credentials — the app talks to systems you configure, and how those are secured is outside its control |
| **Also out of scope** | The unsigned installer warning. That is expected and documented in the [README](README.md#install); it is not a vulnerability report |

## What the app does with your data

Messages, credentials, and search queries stay on your machine. What does leave it, and what
is retained, is described in the [Privacy Notice](legal/PRIVACY.md).

## Disclosure

Please give a reasonable window to ship a fix before publishing details. Once a fix is
released, credit is given in the advisory unless you prefer otherwise.
