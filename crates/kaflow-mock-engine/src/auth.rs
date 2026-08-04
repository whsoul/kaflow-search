//! `AuthApi` mock impl — 모든 인증 동작 즉시 OK.

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

    /// mock 은 사용자의 로컬 자격증명 파일을 건드리지 않는다 — 데모 빌드가 실 키를
    /// 읽어들이면 놀랄 일이다. 빈 목록 = 화면은 "직접 입력" 경로로 폴백한다.
    async fn list_aws_profiles(
        &self,
        _path: Option<String>,
    ) -> Result<Vec<AwsProfileSummary>, EngineError> {
        Ok(vec![])
    }

    /// mock 은 로컬 파일을 보지 않으므로 경로도 비워 둔다 (화면은 출처 표시를 감춘다).
    async fn resolve_aws_paths(&self) -> AwsPaths {
        AwsPaths::default()
    }

    async fn load_aws_profile(
        &self,
        _path: Option<String>,
        _profile: String,
    ) -> Result<AwsProfileCredentials, EngineError> {
        Err(EngineError::InvalidArgument(
            "mock 엔진은 AWS 자격증명 파일을 읽지 않습니다 — 키를 직접 입력하세요".to_string(),
        ))
    }
}
