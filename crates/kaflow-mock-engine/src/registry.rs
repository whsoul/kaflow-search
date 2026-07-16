//! `RegistryApi` mock impl.
//!
//! **CRUD(list/save/delete)는 실제로 동작한다** — `MockStore` 의 in-memory 상태(세션 한정).
//! 등록하면 목록에 남고 삭제하면 사라진다. (실 엔진은 `~/.kaflow/schema_registries.json` 영속.)
//!
//! 반면 **연결 테스트·스키마 조회(test/subjects/fetch/index)는 네트워크(HTTP)** 라 mock 은 stub.
//! mock 의 존재 이유가 "네트워크/인프라 없이 로컬 fixture 로 UX 재현" 이므로 여기까지가 경계다.
//! (실제 Avro/Protobuf 디코딩·registry 참조 해석은 private engine 자산 — `library_split_design.md`.)

use async_trait::async_trait;
use kaflow_api_traits::engine::RegistryApi;
use kaflow_api_traits::error::EngineError;
use kaflow_api_types::{
    RegistryResource, RegistrySchemaIndexEntry, RegistrySchemaView, RegistryTestResult,
};

use crate::MockEngine;

#[async_trait]
impl RegistryApi for MockEngine {
    async fn list_registry_resources(&self) -> Result<Vec<RegistryResource>, EngineError> {
        Ok(self.store.list_registries())
    }

    async fn save_registry_resource(&self, resource: RegistryResource) -> Result<(), EngineError> {
        self.store.save_registry(resource);
        Ok(())
    }

    async fn delete_registry_resource(&self, id: &str) -> Result<(), EngineError> {
        self.store.delete_registry(id);
        Ok(())
    }

    async fn test_registry_resource(
        &self,
        _url: &str,
        _basic_auth: Option<&str>,
    ) -> Result<RegistryTestResult, EngineError> {
        // 연결 테스트는 실제 HTTP 호출 — mock 은 네트워크를 쓰지 않으므로 안내만.
        Ok(RegistryTestResult {
            ok: false,
            message:
                "The mock demo does not run a real connection test (works in production builds)"
                    .to_string(),
            subject_count: None,
        })
    }

    async fn list_registry_subjects(
        &self,
        _url: &str,
        _basic_auth: Option<&str>,
    ) -> Result<Vec<String>, EngineError> {
        Ok(Vec::new())
    }

    async fn fetch_registry_subject_latest(
        &self,
        _url: &str,
        _basic_auth: Option<&str>,
        subject: &str,
    ) -> Result<RegistrySchemaView, EngineError> {
        Ok(RegistrySchemaView {
            subject: subject.to_string(),
            schema_id: 0,
            version: 0,
            schema_type: None,
            schema_text: "// mock engine — no schema".to_string(),
        })
    }

    async fn list_registry_schema_index(
        &self,
        _url: &str,
        _basic_auth: Option<&str>,
    ) -> Result<Vec<RegistrySchemaIndexEntry>, EngineError> {
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    // async trait 래퍼는 store CRUD 를 그대로 위임하므로, tokio 없이 store 를 직접 검사한다
    // (기존 mock 테스트와 동일한 동기 방식 — crate 의 인프라 의존 0 유지).
    use crate::data::MockStore;
    use kaflow_api_types::RegistryResource;

    fn res(id: &str, name: &str) -> RegistryResource {
        RegistryResource {
            id: id.to_string(),
            name: name.to_string(),
            url: "http://localhost:8081".to_string(),
            basic_auth: None,
        }
    }

    #[test]
    fn crud_roundtrip() {
        let store = MockStore::default();
        assert!(store.list_registries().is_empty());

        store.save_registry(res("a", "staging"));
        store.save_registry(res("b", "local"));
        let list = store.list_registries();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].name, "local"); // name 오름차순 (local < staging)

        // 같은 id upsert → 개수 불변, 값 갱신
        store.save_registry(res("a", "prod"));
        let list = store.list_registries();
        assert_eq!(list.len(), 2);
        assert!(list.iter().any(|r| r.id == "a" && r.name == "prod"));

        store.delete_registry("a");
        let list = store.list_registries();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, "b");
    }
}
