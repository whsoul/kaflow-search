//! Kafka 인증 API — bootstrap 별 KafkaAuth 메모리 캐시 관리 + 사전 검증.

use async_trait::async_trait;
use kaflow_api_types::{AwsPaths, AwsProfileCredentials, AwsProfileSummary, KafkaAuth};

use crate::error::EngineError;

#[async_trait]
pub trait AuthApi: Send + Sync {
    /// ConnectScreen 이 연결 직전에 호출 — 메모리 캐시에 인증 등록.
    /// password 등 비밀 정보는 메모리에만 머무르며 디스크에 쓰지 않는다.
    async fn register_kafka_auth(
        &self,
        bootstrap: &str,
        auth: KafkaAuth,
    ) -> Result<(), EngineError>;

    /// 등록된 인증 정보를 메모리에서 제거 (연결 해제 / 비상 정리).
    async fn clear_kafka_auth(&self, bootstrap: &str) -> Result<(), EngineError>;

    /// 자격증명 사전 검증 — raw TCP SaslHandshake / SaslAuthenticate 1회 수행.
    /// `KafkaAuth::None` (PLAINTEXT) 이면 즉시 OK.
    async fn verify_kafka_auth(&self, bootstrap: &str) -> Result<(), EngineError>;

    /// 브로커(리스너)가 지원하는 SASL 메커니즘 목록 조회 — 자격증명 불필요.
    /// `auth` 는 TLS 설정(CA 등)에만 쓰이며 인증은 수행하지 않는다.
    /// register 전(폼 입력 중) 호출되므로 auth 를 인자로 받는다.
    async fn list_sasl_mechanisms(
        &self,
        bootstrap: &str,
        auth: KafkaAuth,
    ) -> Result<Vec<String>, EngineError>;

    /// AWS 자격증명 파일의 프로파일 **목록** 조회 (비밀 제외).
    /// `path=None` 이면 표준 위치(`~/.aws/credentials` + region 은 `~/.aws/config`).
    /// 파일이 없으면 Err — 호출자는 "직접 입력" 경로로 자연스럽게 폴백한다.
    async fn list_aws_profiles(
        &self,
        path: Option<String>,
    ) -> Result<Vec<AwsProfileSummary>, EngineError>;

    /// 엔진이 **실제로 보는** AWS 파일 경로 (환경변수 해석 반영). 화면의 출처 표시용.
    /// 파일 접근을 하지 않으므로 실패하지 않는다 — 구할 수 없으면 빈 문자열.
    async fn resolve_aws_paths(&self) -> AwsPaths;

    /// 선택된 프로파일 하나의 자격증명(비밀 포함)을 읽어 돌려준다.
    /// 화면 폼을 채우는 용도 — 엔진은 이 값을 저장하지 않는다.
    async fn load_aws_profile(
        &self,
        path: Option<String>,
        profile: String,
    ) -> Result<AwsProfileCredentials, EngineError>;
}
