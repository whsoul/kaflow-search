//! `RegistryApi` mock impl.
//!
//! Adding, listing and removing registries works properly — only until the process ends,
//! but properly. Anything that would reach a registry over the network does not, which is
//! where an engine that needs no network necessarily stops.

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
        // Testing a connection means making one. Say so rather than pretending.
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
    // The store is tested directly: the trait methods only hand over to it, and reaching
    // for an async runtime to check that would give this crate the dependency it avoids.
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
        assert_eq!(list[0].name, "local"); // ordered by name

        // Saving under an existing id replaces rather than adds.
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
