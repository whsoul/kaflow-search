//! Schema Registry 리소스 API — `~/.kaflow/schema_registries.json` CRUD + 연결 테스트.
//!
//! deserializer 의 `*ConfluentRegistryRef` 변종이 참조하는 리소스를 관리한다.

use async_trait::async_trait;
use kaflow_api_types::{
    RegistryResource, RegistrySchemaIndexEntry, RegistrySchemaView, RegistryTestResult,
};

use crate::error::EngineError;

#[async_trait]
pub trait RegistryApi: Send + Sync {
    /// 등록된 Schema Registry 리소스 목록.
    async fn list_registry_resources(&self) -> Result<Vec<RegistryResource>, EngineError>;

    /// 리소스 저장 / 갱신 (id 기준 upsert).
    async fn save_registry_resource(&self, resource: RegistryResource) -> Result<(), EngineError>;

    /// 리소스 삭제 (id 기준).
    async fn delete_registry_resource(&self, id: &str) -> Result<(), EngineError>;

    /// 연결 테스트 — `GET <url>/subjects`. 연결 실패는 에러가 아니라 `RegistryTestResult.ok=false`.
    async fn test_registry_resource(
        &self,
        url: &str,
        basic_auth: Option<&str>,
    ) -> Result<RegistryTestResult, EngineError>;

    /// 등록 화면의 스키마 브라우저 — `GET <url>/subjects` (subject 이름 목록).
    async fn list_registry_subjects(
        &self,
        url: &str,
        basic_auth: Option<&str>,
    ) -> Result<Vec<String>, EngineError>;

    /// 선택한 subject 의 최신 스키마 상세 — `GET <url>/subjects/{subject}/versions/latest`.
    async fn fetch_registry_subject_latest(
        &self,
        url: &str,
        basic_auth: Option<&str>,
        subject: &str,
    ) -> Result<RegistrySchemaView, EngineError>;

    /// 전체 스키마 인덱스(한 방) — `GET <url>/schemas?latestOnly=true`. id/type + 참조 그래프용.
    async fn list_registry_schema_index(
        &self,
        url: &str,
        basic_auth: Option<&str>,
    ) -> Result<Vec<RegistrySchemaIndexEntry>, EngineError>;
}
