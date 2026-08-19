# Kaflow Search Privacy Notice

**Version: 1.0-beta**
**Last Updated: 2026-08-11**

This Privacy Notice describes what information Kaflow Search (the "Software") processes, what is transmitted off your device, and what the Licensor (Whsoul Tools) receives and retains.

Kaflow Search is a desktop application. Its core design principle is that **your Kafka data stays on your device.**

## 1. WHAT STAYS LOCAL (NOT TRANSMITTED TO THE LICENSOR)

The following data is processed and stored only on your device and is never transmitted to the Licensor:

- Kafka message content (keys, payloads, headers) and topic names;
- local search indexes and index metadata;
- search queries and search results;
- cluster addresses, connection settings, and schema registry settings;
- authentication credentials (the Software does not persist passwords to disk; they are re-entered per session);
- exported files and backups;
- application settings.

The Software transmits your Kafka data only to the Kafka clusters and schema registries **that you yourself configure**, as required to perform the functions you invoke. Those systems are yours (or your organization's); their handling of data is outside the Licensor's control.

## 2. COMPATIBILITY CHECK (APP-CHECK) REQUESTS

When started (and periodically while running), the Software contacts the Licensor's online service to check version compatibility and service policy and to retrieve informational content. Each request contains only the following limited technical information:

| Field | What it is | Purpose |
|---|---|---|
| `appInstanceId` | A random UUID generated locally at first run. Contains no personal data. Reset when application data is deleted. | Correlating requests; rate limiting; abuse detection |
| `deviceIdHash` | A SHA-256 hash of the operating system's machine identifier. The raw identifier never leaves your device. | Abuse detection (e.g., detecting duplicated or cloned installations) |
| `appVersion` | The Software version | Update and compatibility decisions |
| `platform`, `arch` | Operating system and CPU architecture | Compatibility decisions |
| `locale` | The UI language | Localizing service messages |

The Software does not send your IP address. As with any request over the internet, the hosting provider necessarily sees the connecting IP address in order to route and answer the request; the service does not record, hash, or store it.

These requests do not include Kafka message content, topic names, cluster addresses, credentials, search queries, or any content of your work.

**Retention:** the Licensor does not store these requests in a database. They appear only in the hosting platform's operational logs, which that platform keeps for a short period — days, not months — and then discards. The Licensor does not copy them anywhere else.

## 3. DIAGNOSTIC REPORTS (USER-INITIATED ONLY)

The Software can generate a diagnostic report when you request one. Reports are created and stored **locally** on your device. They contain technical reproduction context (application version, feature flags, timing, error categories, detected Kafka broker version) and are designed to exclude message content.

A diagnostic report reaches the Licensor **only if you choose to send or share it yourself.** You can inspect the report file before sharing it.

## 4. FUTURE FEATURES

If features requiring additional data processing are introduced in the future (for example, separately licensed features), the additional processing will be described in an updated version of this Notice before those features apply to you.

## 5. WHAT THE LICENSOR DOES NOT DO

- No advertising, ad tracking, or sale of data.
- No behavioral analytics or usage telemetry beyond the compatibility checks described in Section 2.
- No collection of Kafka message content, topic names, cluster addresses, credentials, or search queries.

## 6. SERVICE PROVIDERS AND DISCLOSURE

The Licensor uses infrastructure service providers (such as hosting and content delivery) to operate the online services described above; they process data on the Licensor's behalf. Information may also be disclosed where required by law.

The online services run on a globally distributed edge platform: each request is handled at the location nearest to you rather than in one fixed region. Where these services store data persistently, that data is held in the Asia-Pacific (APAC) region and is not replicated to other regions.

## 7. YOUR RIGHTS

Depending on applicable law (including the Personal Information Protection Act of the Republic of Korea), you may have rights to request access to, correction of, or deletion of information associated with your identifiers.

Because the Licensor does not store the requests described in Section 2, there is normally no record held against your identifiers to access, correct, or delete. These rights apply to any information the Licensor does hold — for example, a diagnostic report you chose to share, or a message you sent to the address below.

Contact: legal@whsoul-tools.com

## 8. CHILDREN

The Software is a professional developer tool and is not directed to children.

## 9. CHANGES TO THIS NOTICE

The Licensor may update this Privacy Notice. Material changes will be indicated by updating the version and date above, and, where reasonably practicable, notified within the Software. The current version is available at https://github.com/whsoul/kaflow-search/blob/main/legal/PRIVACY.md.

## 10. CONTACT

Data controller: Whsoul Tools

Email: legal@whsoul-tools.com

Website: https://whsoul-tools.com

### Business information

Trade name: Whsoul Tools

_Kaflow Search is a product of Whsoul Tools._
