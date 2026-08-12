//! Saved schema registries — the ones deserializers refer to by id.

use async_trait::async_trait;
use kaflow_api_types::{
    RegistryResource, RegistrySchemaIndexEntry, RegistrySchemaView, RegistryTestResult,
};

use crate::error::EngineError;

#[async_trait]
pub trait RegistryApi: Send + Sync {
    /// The saved registries.
    async fn list_registry_resources(&self) -> Result<Vec<RegistryResource>, EngineError>;

    /// Saves or updates one, matched by id.
    async fn save_registry_resource(&self, resource: RegistryResource) -> Result<(), EngineError>;

    /// Removes one.
    async fn delete_registry_resource(&self, id: &str) -> Result<(), EngineError>;

    /// Tries a registry out. ⚠️ **A registry that cannot be reached is a result, not an
    /// error** — the caller asked whether it works, and "no" is an answer.
    async fn test_registry_resource(
        &self,
        url: &str,
        basic_auth: Option<&str>,
    ) -> Result<RegistryTestResult, EngineError>;

    /// The subject names a registry holds.
    async fn list_registry_subjects(
        &self,
        url: &str,
        basic_auth: Option<&str>,
    ) -> Result<Vec<String>, EngineError>;

    /// One subject's latest schema.
    async fn fetch_registry_subject_latest(
        &self,
        url: &str,
        basic_auth: Option<&str>,
        subject: &str,
    ) -> Result<RegistrySchemaView, EngineError>;

    /// Every schema at once — enough to work out how they refer to one another.
    async fn list_registry_schema_index(
        &self,
        url: &str,
        basic_auth: Option<&str>,
    ) -> Result<Vec<RegistrySchemaIndexEntry>, EngineError>;
}
