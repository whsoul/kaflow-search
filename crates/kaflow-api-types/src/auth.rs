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
                ..
            } => StoredAuthConfig {
                protocol: KafkaProtocol::Ssl,
                ca_cert_path: ca_cert_path.clone(),
                client_cert_path: client_cert_path.clone(),
                client_key_path: client_key_path.clone(),
                ..Default::default()
            },
            KafkaAuth::SaslSsl {
                mechanism,
                username,
                aws_region,
                ca_cert_path,
                client_cert_path,
                client_key_path,
                ..
            } => StoredAuthConfig {
                protocol: KafkaProtocol::SaslSsl,
                sasl_mechanism: Some(*mechanism),
                username: Some(username.clone()),
                aws_region: aws_region.clone(),
                ca_cert_path: ca_cert_path.clone(),
                client_cert_path: client_cert_path.clone(),
                client_key_path: client_key_path.clone(),
                // Where the credentials came from is not part of the connection itself —
                // by then they are just values. Only the caller that read them knows, so
                // it supplies these separately.
                ..Default::default()
            },
        }
    }
}
