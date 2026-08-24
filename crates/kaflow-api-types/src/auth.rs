//! How a cluster is connected to, and what of that may be kept.
//!
//! The split between the two types here is a rule, not a convenience. `KafkaAuth` carries
//! secrets and exists only for the length of a connection attempt; `StoredAuthConfig` is
//! the one of the two that may be written down — protocol, mechanism, username,
//! certificate paths.
//!
//! ⚠️ **An implementation must never persist a password, in either form.** It is to be
//! asked for on each connection instead. The type split makes that easy to honour but does
//! not enforce it: `KafkaAuth` is handed over whole, and nothing here stops it being
//! written somewhere.

use serde::{Deserialize, Serialize};

/// Which security protocol a connection uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KafkaProtocol {
    #[serde(rename = "PLAINTEXT")]
    Plaintext,
    #[serde(rename = "SASL_PLAINTEXT")]
    SaslPlaintext,
    #[serde(rename = "SSL")]
    Ssl,
    #[serde(rename = "SASL_SSL")]
    SaslSsl,
}

impl Default for KafkaProtocol {
    fn default() -> Self {
        KafkaProtocol::Plaintext
    }
}

/// Which SASL mechanism authenticates the connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SaslMechanism {
    #[serde(rename = "PLAIN")]
    Plain,
    #[serde(rename = "SCRAM-SHA-256")]
    ScramSha256,
    #[serde(rename = "SCRAM-SHA-512")]
    ScramSha512,
    /// Standard OAUTHBEARER (RFC 7628). The token travels in `password`; `username` is
    /// unused.
    #[serde(rename = "OAUTHBEARER")]
    OAuthBearer,
    /// AWS MSK IAM. On the wire this is OAUTHBEARER, but the token is signed per
    /// connection rather than supplied: `username` is the access key id, `password` the
    /// secret, and a region is required. TLS is mandatory, so only `SASL_SSL` applies.
    #[serde(rename = "AWS_MSK_IAM")]
    AwsMskIam,
}

impl SaslMechanism {
    /// The identifier as it goes on the wire.
    pub fn wire_name(&self) -> &'static str {
        match self {
            SaslMechanism::Plain => "PLAIN",
            SaslMechanism::ScramSha256 => "SCRAM-SHA-256",
            SaslMechanism::ScramSha512 => "SCRAM-SHA-512",
            SaslMechanism::OAuthBearer => "OAUTHBEARER",
            SaslMechanism::AwsMskIam => "OAUTHBEARER",
        }
    }
}

/// Everything needed to connect, secrets included.
///
/// **Must not be stored.** Converting to `StoredAuthConfig` drops the secrets, and that
/// conversion has to be the only route from here to disk.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "protocol")]
pub enum KafkaAuth {
    #[serde(rename = "PLAINTEXT")]
    None,
    #[serde(rename = "SASL_PLAINTEXT")]
    SaslPlaintext {
        mechanism: SaslMechanism,
        username: String,
        password: String,
    },
    #[serde(rename = "SSL")]
    Ssl {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        ca_cert_path: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        client_cert_path: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        client_key_path: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        key_password: Option<String>,
        /// SHA-256 fingerprints of server certificates the user has explicitly chosen to
        /// trust despite failing chain validation (expired, unknown issuer, unsupported
        /// version, ...). Not a secret — a broker's certificate is public.
        ///
        /// An implementation must accept a presented certificate whose fingerprint is in
        /// this list even when standard chain validation would reject it, while still
        /// requiring proof of possession of the matching private key. It must not relax
        /// any other check for a fingerprint not in this list.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pinned_cert_fingerprints: Vec<String>,
    },
    #[serde(rename = "SASL_SSL")]
    SaslSsl {
        mechanism: SaslMechanism,
        username: String,
        password: String,
        /// Required for AWS MSK IAM; ignored otherwise.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        aws_region: Option<String>,
        /// Set when the credentials are temporary.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        aws_session_token: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        ca_cert_path: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        client_cert_path: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        client_key_path: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        key_password: Option<String>,
        /// See `KafkaAuth::Ssl::pinned_cert_fingerprints` — same contract.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pinned_cert_fingerprints: Vec<String>,
    },
}

impl Default for KafkaAuth {
    fn default() -> Self {
        KafkaAuth::None
    }
}

impl KafkaAuth {
    pub fn protocol(&self) -> KafkaProtocol {
        match self {
            KafkaAuth::None => KafkaProtocol::Plaintext,
            KafkaAuth::SaslPlaintext { .. } => KafkaProtocol::SaslPlaintext,
            KafkaAuth::Ssl { .. } => KafkaProtocol::Ssl,
            KafkaAuth::SaslSsl { .. } => KafkaProtocol::SaslSsl,
        }
    }

    /// The reason this combination cannot work, if it cannot.
    ///
    /// Only one is refused: AWS MSK IAM without TLS, which the service does not accept
    /// in the first place. Everything else is supported.
    pub fn unsupported_error(&self) -> Option<String> {
        match self {
            KafkaAuth::SaslPlaintext {
                mechanism: SaslMechanism::AwsMskIam,
                ..
            } => Some(
                "AWS MSK IAM requires TLS — select SASL_SSL as the security protocol".to_string(),
            ),
            KafkaAuth::None
            | KafkaAuth::Ssl { .. }
            | KafkaAuth::SaslPlaintext { .. }
            | KafkaAuth::SaslSsl { .. } => None,
        }
    }
}

/// A summary of one AWS profile, without its secrets.
///
/// Listing profiles must not hand over every secret on the machine — the secrets of a
/// single profile are read only once one has been chosen.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AwsProfileSummary {
    pub name: String,
    /// The access key id, which identifies rather than authenticates. `None` where the
    /// profile has none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub access_key_id: Option<String>,
    /// The region found for this profile, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
    /// Whether the credentials are temporary and need a session token.
    pub has_session_token: bool,
    /// A profile that defines only a role or sign-in, with no key to use directly.
    pub sso_only: bool,
}

/// The AWS credential file paths actually in use.
///
/// ⚠️ **These must be taken from the engine, not assembled by the caller.** Environment
/// variables can move either file, so a path built from the usual `~/.aws/...` would tell
/// the user it read somewhere it did not.
///
/// Empty where no path could be worked out. Existence is not checked — this says where it
/// would look, not what is there.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AwsPaths {
    /// Where credentials are read from unless another file was chosen.
    pub credentials: String,
    /// Consulted only for a region.
    pub config: String,
}

/// One profile's actual credentials, secrets included.
///
/// **Not to be stored.** This only saves the user copying values out of a file by hand;
/// what may be kept is unchanged.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AwsProfileCredentials {
    pub access_key_id: String,
    pub secret_access_key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_token: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
}

/// What may be written to disk: everything except the secrets.
///
/// A password is asked for on each connection and combined with this to form a
/// `KafkaAuth` in memory.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct StoredAuthConfig {
    pub protocol: KafkaProtocol,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sasl_mechanism: Option<SaslMechanism>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    /// The region for AWS MSK IAM. Not a secret, so it is kept.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub aws_region: Option<String>,
    /// Which credential file to read next time — a location, not a credential.
    ///
    /// Temporary credentials expire within hours and the file is rewritten when they are
    /// renewed. Remembering where to look means reconnecting picks up the current values
    /// on its own, while still storing none of them. Empty means the standard location.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub aws_credentials_file: Option<String>,
    /// Which profile in that file. Not a secret.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub aws_profile: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ca_cert_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_cert_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_key_path: Option<String>,
    /// See `KafkaAuth::Ssl::pinned_cert_fingerprints`. Not a secret, so it is kept —
    /// carrying it over on reconnect is the point (SSH `known_hosts`-style trust).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pinned_cert_fingerprints: Vec<String>,
}

impl StoredAuthConfig {
    /// Drops the secrets, leaving what may be stored.
    pub fn from_auth(auth: &KafkaAuth) -> Self {
        match auth {
            KafkaAuth::None => StoredAuthConfig {
                protocol: KafkaProtocol::Plaintext,
                ..Default::default()
            },
            KafkaAuth::SaslPlaintext {
                mechanism,
                username,
                ..
            } => StoredAuthConfig {
                protocol: KafkaProtocol::SaslPlaintext,
                sasl_mechanism: Some(*mechanism),
                username: Some(username.clone()),
                ..Default::default()
            },
            KafkaAuth::Ssl {
                ca_cert_path,
                client_cert_path,
                client_key_path,
                pinned_cert_fingerprints,
                ..
            } => StoredAuthConfig {
                protocol: KafkaProtocol::Ssl,
                ca_cert_path: ca_cert_path.clone(),
                client_cert_path: client_cert_path.clone(),
                client_key_path: client_key_path.clone(),
                pinned_cert_fingerprints: pinned_cert_fingerprints.clone(),
                ..Default::default()
            },
            KafkaAuth::SaslSsl {
                mechanism,
                username,
                aws_region,
                ca_cert_path,
                client_cert_path,
                client_key_path,
                pinned_cert_fingerprints,
                ..
            } => StoredAuthConfig {
                protocol: KafkaProtocol::SaslSsl,
                sasl_mechanism: Some(*mechanism),
                username: Some(username.clone()),
                aws_region: aws_region.clone(),
                ca_cert_path: ca_cert_path.clone(),
                client_cert_path: client_cert_path.clone(),
                client_key_path: client_key_path.clone(),
                pinned_cert_fingerprints: pinned_cert_fingerprints.clone(),
                // Where the credentials came from is not part of the connection itself —
                // by then they are just values. Only the caller that read them knows, so
                // it supplies these separately.
                ..Default::default()
            },
        }
    }
}

/// Why a presented TLS certificate failed standard chain validation.
///
/// A locale-invariant identifier — a client translates it for display; the variant name
/// itself must not change (it is compared as text: FE contracts, tests, ...).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CertFailureReason {
    /// Not a v3 X.509 certificate.
    UnsupportedVersion,
    /// No trusted CA issued it (includes self-signed).
    UnknownIssuer,
    Expired,
    NotValidYet,
    /// The certificate's names do not cover the address that was dialed.
    HostnameMismatch,
    /// The issuing CA has explicitly revoked it.
    ///
    /// An implementation must never let a caller bypass validation for this reason.
    Revoked,
    /// Any other validation failure not covered above.
    Other,
}

/// A server certificate that failed standard validation, offered up for the caller to
/// decide whether to trust it anyway (see `CertFailureReason::Revoked` for the one case
/// where an implementation must not offer that choice).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UntrustedCertInfo {
    /// The broker address (`host:port`) that presented this certificate.
    pub broker_addr: String,
    /// SHA-256 fingerprint of the certificate, lowercase hex, colon-separated
    /// (`"aa:bb:cc:..."`) — the form a caller would pin back via
    /// `pinned_cert_fingerprints` to trust this exact certificate.
    pub sha256_fingerprint: String,
    pub reason: CertFailureReason,
    /// The underlying validation error, English, for display alongside the reason —
    /// not meant to be parsed.
    pub detail: String,
}

/// A broker that could not be reached at all while checking cluster-wide certificate
/// trust — not a certificate problem, just no answer from that address.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnreachableBrokerInfo {
    /// The broker address (`host:port`) that did not respond.
    pub broker_addr: String,
    /// The underlying connection error, English, for display alongside the address —
    /// not meant to be parsed.
    pub detail: String,
}

/// What `verify_kafka_auth`/`confirm_kafka_cert_trust`/`verify_cluster_broker_trust` resolve
/// to, over the wire.
///
/// A tagged enum rather than a plain error: `UntrustedCert` is a decision point for the
/// caller, not a failure — only genuine failures (network, wrong password, ...) still
/// travel as an error.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum VerifyKafkaAuthResult {
    Trusted {
        /// Brokers that could not be reached at all while checking trust — not a
        /// reason to fail, but a caller should surface it rather than discard it.
        /// Always empty for a check scoped to a single already-connected address.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        unreachable_brokers: Vec<UnreachableBrokerInfo>,
    },
    UntrustedCert {
        candidates: Vec<UntrustedCertInfo>,
    },
}
