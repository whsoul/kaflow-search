//! Authentication that always succeeds, since there is nothing to authenticate against.

use async_trait::async_trait;
use kaflow_api_traits::engine::AuthApi;
use kaflow_api_traits::error::EngineError;
use kaflow_api_types::{AwsPaths, AwsProfileCredentials, AwsProfileSummary, KafkaAuth};

use crate::MockEngine;

#[async_trait]
impl AuthApi for MockEngine {
    async fn register_kafka_auth(
        &self,
        _bootstrap: &str,
        _auth: KafkaAuth,
    ) -> Result<(), EngineError> {
        Ok(())
    }

    async fn clear_kafka_auth(&self, _bootstrap: &str) -> Result<(), EngineError> {
        Ok(())
    }

    async fn verify_kafka_auth(&self, _bootstrap: &str) -> Result<(), EngineError> {
        Ok(())
    }

    async fn list_sasl_mechanisms(
        &self,
        _bootstrap: &str,
        _auth: KafkaAuth,
    ) -> Result<Vec<String>, EngineError> {
        Ok(vec![
            "PLAIN".to_string(),
            "SCRAM-SHA-256".to_string(),
            "SCRAM-SHA-512".to_string(),
        ])
    }

    /// **Reads none of the user's credential files.** A demonstration build that helped
    /// itself to real keys would be a surprise, and not a welcome one. The empty list
    /// leaves the caller to ask for them directly.
    async fn list_aws_profiles(
        &self,
        _path: Option<String>,
    ) -> Result<Vec<AwsProfileSummary>, EngineError> {
        Ok(vec![])
    }

    /// Empty, for the same reason: no file was looked at, so naming one would be untrue.
    async fn resolve_aws_paths(&self) -> AwsPaths {
        AwsPaths::default()
    }

    async fn load_aws_profile(
        &self,
        _path: Option<String>,
        _profile: String,
    ) -> Result<AwsProfileCredentials, EngineError> {
        Err(EngineError::InvalidArgument(
            "This engine does not read AWS credential files — enter the keys directly".to_string(),
        ))
    }
}
