//! Authentication commands. Each one hands straight to the engine.

use kaflow_api_traits::error::EngineError;
use kaflow_api_traits::KafkaToolEngine;
use kaflow_api_types::auth::UnreachableBrokerInfo;
use kaflow_api_types::{
    AwsPaths, AwsProfileCredentials, AwsProfileSummary, KafkaAuth, VerifyKafkaAuthResult,
};
use std::sync::Arc;

/// Shared by `verify_kafka_auth`/`confirm_kafka_cert_trust`: a bare `EngineError` result
/// only distinguishes success from failure, but `UntrustedCert` is a decision the caller
/// needs to act on, not a failure — so it travels back as a resolved `VerifyKafkaAuthResult`
/// instead of a rejected promise. Every other error keeps the existing `into_string` shape.
/// These two never discover unreachable brokers of their own (single already-known address).
fn map_verify_result(result: Result<(), EngineError>) -> Result<VerifyKafkaAuthResult, String> {
    map_verify_result_with_diagnostics(result.map(|()| Vec::new()))
}

/// `verify_cluster_broker_trust`'s variant: success carries the addresses that were
/// unreachable during the check, which the caller should still be told about.
fn map_verify_result_with_diagnostics(
    result: Result<Vec<UnreachableBrokerInfo>, EngineError>,
) -> Result<VerifyKafkaAuthResult, String> {
    match result {
        Ok(unreachable_brokers) => Ok(VerifyKafkaAuthResult::Trusted {
            unreachable_brokers,
        }),
        Err(EngineError::UntrustedCert(candidates)) => {
            Ok(VerifyKafkaAuthResult::UntrustedCert { candidates })
        }
        Err(e) => Err(e.into_string()),
    }
}

#[tauri::command]
pub async fn register_kafka_auth(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    bootstrap: String,
    auth: KafkaAuth,
) -> Result<(), String> {
    engine
        .register_kafka_auth(&bootstrap, auth)
        .await
        .map_err(|e| e.into_string())
}

#[tauri::command]
pub async fn clear_kafka_auth(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    bootstrap: String,
) -> Result<(), String> {
    engine
        .clear_kafka_auth(&bootstrap)
        .await
        .map_err(|e| e.into_string())
}

#[tauri::command]
pub async fn verify_kafka_auth(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    bootstrap: String,
) -> Result<VerifyKafkaAuthResult, String> {
    map_verify_result(engine.verify_kafka_auth(&bootstrap).await)
}

#[tauri::command]
pub async fn verify_cluster_broker_trust(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    bootstrap: String,
) -> Result<VerifyKafkaAuthResult, String> {
    map_verify_result_with_diagnostics(engine.verify_cluster_broker_trust(&bootstrap).await)
}

#[tauri::command]
pub async fn confirm_kafka_cert_trust(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    bootstrap: String,
    accepted_fingerprints: Vec<String>,
) -> Result<VerifyKafkaAuthResult, String> {
    map_verify_result(
        engine
            .confirm_kafka_cert_trust(&bootstrap, accepted_fingerprints)
            .await,
    )
}

#[tauri::command]
pub async fn list_sasl_mechanisms(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    bootstrap: String,
    auth: KafkaAuth,
) -> Result<Vec<String>, String> {
    engine
        .list_sasl_mechanisms(&bootstrap, auth)
        .await
        .map_err(|e| e.into_string())
}

#[tauri::command]
pub async fn list_aws_profiles(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    path: Option<String>,
) -> Result<Vec<AwsProfileSummary>, String> {
    engine
        .list_aws_profiles(path)
        .await
        .map_err(|e| e.into_string())
}

#[tauri::command]
pub async fn resolve_aws_paths(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
) -> Result<AwsPaths, String> {
    Ok(engine.resolve_aws_paths().await)
}

#[tauri::command]
pub async fn load_aws_profile(
    engine: tauri::State<'_, Arc<dyn KafkaToolEngine>>,
    path: Option<String>,
    profile: String,
) -> Result<AwsProfileCredentials, String> {
    engine
        .load_aws_profile(path, profile)
        .await
        .map_err(|e| e.into_string())
}
