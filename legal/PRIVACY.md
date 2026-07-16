# Kaflow Search Privacy Notice

**Version: 1.0-beta**
**Last Updated: [Effective Date]**

This Privacy Notice describes what information Kaflow Search (the "Software") processes, what is transmitted off your device, and what the Licensor ([Legal Name], operating under the name "Kaflow Search") receives and retains.

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
| `deviceIdHash` | A SHA-256 hash of the operating system's machine identifier. The raw identifier never leaves your device and cannot be recovered from the hash. | Abuse detection (e.g., detecting duplicated or cloned installations) |
| `appVersion` | The Software version | Update and compatibility decisions |
| `platform`, `arch` | Operating system and CPU architecture | Compatibility decisions |
| `locale` | The UI language | Localizing service messages |

In addition, the service derives a **hashed form of the request IP address** on the server side (the Software does not send it). Raw IP addresses are processed transiently to serve the request.

These requests do not include Kafka message content, topic names, cluster addresses, credentials, search queries, or any content of your work.

**Retention:** requests are retained as append-only service logs for service operation, security, and abuse prevention, for up to [retention period, e.g., 24 months], and are then deleted or irreversibly aggregated.

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

Server infrastructure is located in [server region(s)]. If you access the services from another jurisdiction, the information in Section 2 is transferred to that location.

## 7. YOUR RIGHTS

Depending on applicable law (including the Personal Information Protection Act of the Republic of Korea), you may have rights to request access to, correction of, or deletion of information associated with your identifiers.

Because the identifiers in Section 2 are pseudonymous, you may need to provide your `appInstanceId` (visible in the application) for the Licensor to locate associated records.

Contact: [Contact Email]

## 8. CHILDREN

The Software is a professional developer tool and is not directed to children.

## 9. CHANGES TO THIS NOTICE

The Licensor may update this Privacy Notice. Material changes will be indicated by updating the version and date above, and, where reasonably practicable, notified within the Software. The current version is available at [Privacy Notice URL].

## 10. CONTACT

Data controller: [Legal Name], operating under the name "Kaflow Search"

Email: [Contact Email]

Website: [Official Website]
