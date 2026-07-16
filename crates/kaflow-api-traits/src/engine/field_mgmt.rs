//! Field 관리 API — 토픽 인덱스 필드 추가/제거/재인덱싱 + watch.

use async_trait::async_trait;
use kaflow_api_types::{IndexedFieldInput, TopicDeserializers};

use crate::error::EngineError;

#[async_trait]
pub trait FieldMgmtApi: Send + Sync {
    /// 토픽별 인덱싱 대상 필드 설정.
    async fn set_indexed_fields(
        &self,
        workspace: &str,
        topic: &str,
        fields: Vec<IndexedFieldInput>,
    ) -> Result<(), EngineError>;

    /// 지정 필드를 M-key 들에서 다시 읽어 I-key 재구성. 반환값 = 새로 쓴 I-key 수.
    async fn reindex_fields_from_meta(
        &self,
        workspace: &str,
        topic: &str,
        fields: Vec<String>,
    ) -> Result<usize, EngineError>;

    /// 지정 필드의 I-key 만 일괄 삭제. 반환값 = 삭제된 I-key 수.
    async fn drop_fields_from_index(
        &self,
        workspace: &str,
        topic: &str,
        fields: Vec<String>,
    ) -> Result<usize, EngineError>;

    /// 토픽 자체를 인덱스 / picker 에서 제거 (CF drop). 반환값 = (i_count, m_count).
    async fn remove_topic_from_index(
        &self,
        workspace: &str,
        topic: &str,
    ) -> Result<(usize, usize), EngineError>;

    /// 토픽 picker 에 등록 (T-meta 생성, 인덱스 비어있어도 가능).
    async fn ensure_topic_watched(&self, workspace: &str, topic: &str) -> Result<(), EngineError>;

    /// 토픽의 key/value deserializer 페어를 갱신한다. None 이면 기본 (Json × 2) 으로 되돌림.
    async fn set_topic_deserializers(
        &self,
        workspace: &str,
        topic: &str,
        deserializers: Option<TopicDeserializers>,
    ) -> Result<(), EngineError>;

    /// 인덱싱 **전** 어절(tokenize) 대상 필드 지정 (picker). indexed_fields 직교 —
    /// 전체 필드 인덱싱을 유지하며 지정 필드만 어절. 한도 초과는 InvalidArgument.
    async fn set_tokenize_fields(
        &self,
        workspace: &str,
        topic: &str,
        fields: Vec<String>,
    ) -> Result<(), EngineError>;

    /// picker 에서 토픽 unwatch — 명시적 제거. 반환값 = (i_count, m_count).
    async fn unwatch_topic(
        &self,
        workspace: &str,
        topic: &str,
    ) -> Result<(usize, usize), EngineError>;

    /// size cleanup 으로 자동 정리됨 마크.
    async fn mark_topic_auto_cleaned(
        &self,
        workspace: &str,
        topic: &str,
        timestamp_ms: i64,
    ) -> Result<(), EngineError>;
}
