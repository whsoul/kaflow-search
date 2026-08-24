//! Holding credentials for a connection, and checking them before one is attempted.

use async_trait::async_trait;
use kaflow_api_types::auth::UnreachableBrokerInfo;
use kaflow_api_types::{AwsPaths, AwsProfileCredentials, AwsProfileSummary, KafkaAuth};

use crate::error::EngineError;

#[async_trait]
pub trait AuthApi: Send + Sync {
    /// Registers credentials for a cluster, to be used by later calls.
    ///
    /// **Secrets stay in memory and must never be written to disk from here.**
    async fn register_kafka_auth(
        &self,
        bootstrap: &str,
        auth: KafkaAuth,
    ) -> Result<(), EngineError>;

    /// Forgets them again.
    async fn clear_kafka_auth(&self, bootstrap: &str) -> Result<(), EngineError>;

    /// Tries the credentials once, so a failure is reported before anything else depends
    /// on them. Succeeds immediately where there is nothing to authenticate.
    ///
    /// Where TLS is in use and the presented server certificate fails standard chain
    /// validation, this returns `EngineError::UntrustedCert` rather than a plain failure
    /// where the certificate was captured well enough to identify — see
    /// `confirm_kafka_cert_trust`.
    async fn verify_kafka_auth(&self, bootstrap: &str) -> Result<(), EngineError>;

    /// Re-verifies credentials for `bootstrap` after the caller has reviewed and accepted
    /// specific TLS certificate fingerprints from a prior `EngineError::UntrustedCert`.
    ///
    /// An implementation must accept a presented certificate only when its fingerprint is
    /// among `accepted_fingerprints`, and must still require proof of possession of the
    /// matching private key — this is not a blanket bypass of certificate verification.
    /// The accepted fingerprints are remembered for `bootstrap` beyond this one call, the
    /// same as `register_kafka_auth`'s credentials.
    async fn confirm_kafka_cert_trust(
        &self,
        bootstrap: &str,
        accepted_fingerprints: Vec<String>,
    ) -> Result<(), EngineError>;

    /// Verifies certificate trust for connection endpoints discovered from the cluster
    /// itself, not only `bootstrap`.
    ///
    /// Fails the same way `verify_kafka_auth` does, including `EngineError::UntrustedCert`,
    /// resolved the same way through `confirm_kafka_cert_trust`. On success, returns the
    /// brokers that could not be reached at all — not itself a failure, but a caller
    /// should surface it rather than discard it silently.
    async fn verify_cluster_broker_trust(
        &self,
        bootstrap: &str,
    ) -> Result<Vec<UnreachableBrokerInfo>, EngineError>;

    /// Asks the broker which SASL mechanisms it offers.
    ///
    /// No credentials are needed or used — `auth` is taken only for its transport settings,
    /// since this is meant to be callable while the user is still filling the form.
    async fn list_sasl_mechanisms(
        &self,
        bootstrap: &str,
        auth: KafkaAuth,
    ) -> Result<Vec<String>, EngineError>;

    /// Lists the AWS profiles available, without their secrets. `None` reads the standard
    /// location. A missing file is an error, which a caller can treat as "type them in".
    async fn list_aws_profiles(
        &self,
        path: Option<String>,
    ) -> Result<Vec<AwsProfileSummary>, EngineError>;

    /// Where the engine would look, with the environment taken into account. Touches no
    /// file, so it cannot fail; an unknown path comes back empty.
    async fn resolve_aws_paths(&self) -> AwsPaths;

    /// Reads one profile's credentials, secrets included. **The engine stores none of
    /// them** — this only saves the user copying them by hand.
    async fn load_aws_profile(
        &self,
        path: Option<String>,
        profile: String,
    ) -> Result<AwsProfileCredentials, EngineError>;
}
