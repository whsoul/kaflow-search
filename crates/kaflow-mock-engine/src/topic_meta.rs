//! `TopicMetaApi` mock impl — 로컬 fixture(`MockStore`) 기반 read.
//! 토픽 리스트 / 메타 / 메시지 수 / offset 현황 / size profile 은 fixture 에서 derive.
//! 클러스터 토폴로지·버전 등 Kafka 클러스터 특화 정보는 하드코딩 stub.

use std::collections::{BTreeSet, HashMap};

use async_trait::async_trait;
use kaflow_api_traits::engine::TopicMetaApi;
use kaflow_api_traits::error::EngineError;
use kaflow_api_types::{
    BrokerInfo, ClusterTopology, DeserializerSpec, KafkaVersionInfo, PartitionOffsetStatus,
    PartitionTopology, SuggestTokenizeFieldsResponse, TopicConfigInfoResponse, TopicFieldMeta,
    TopicIndexSize, TopicLagStatus, TopicMessageCount, TopicOffsetStatus, TopicSizeProfile,
    TopicTopology, WorkspaceStorageStatus,
};

use crate::data::MockTopic;
use crate::MockEngine;

/// fixture 토픽 → `TopicFieldMeta` (인덱싱 완료 상태로 표시). `tok` = 현재 어절 대상 필드.
fn topic_field_meta(t: &MockTopic, tok: &BTreeSet<String>) -> TopicFieldMeta {
    let mut earliest: HashMap<u32, i64> = HashMap::new();
    let mut latest: HashMap<u32, i64> = HashMap::new();
    for m in &t.messages {
        let off = m.offset as i64;
        earliest
            .entry(m.partition)
            .and_modify(|e| {
                if off < *e {
                    *e = off;
                }
            })
            .or_insert(off);
        latest
            .entry(m.partition)
            .and_modify(|e| {
                if off > *e {
                    *e = off;
                }
            })
            .or_insert(off);
    }
    // kafka high watermark = max indexed offset + 1 (caught-up 표시).
    let kafka_latest: HashMap<u32, i64> = latest.iter().map(|(p, o)| (*p, o + 1)).collect();
    TopicFieldMeta {
        topic: t.name.clone(),
        key_fields: t.key_fields.clone(),
        payload_fields: t.payload_fields.clone(),
        header_fields: Vec::new(),
        message_count: t.messages.len(),
        index_entry_count: t.messages.iter().map(|m| m.fields.len()).sum(),
        indexed: true,
        field_subset: false,
        latest_indexed_offsets: latest,
        earliest_offsets: earliest,
        kafka_latest_offsets: kafka_latest,
        partition_count: Some(t.partition_count as usize),
        last_incremental_sync_at: Some(
            t.messages
                .iter()
                .map(|m| m.ts_millis as i64)
                .max()
                .unwrap_or(0),
        ),
        tokenize_fields: tok.iter().cloned().collect(),
        ..Default::default()
    }
}

#[async_trait]
impl TopicMetaApi for MockEngine {
    async fn list_topic_metas(&self, _workspace: &str) -> Result<Vec<TopicFieldMeta>, EngineError> {
        Ok(self
            .store
            .topics
            .iter()
            .map(|t| topic_field_meta(t, &self.store.tokenized_for(&t.name)))
            .collect())
    }

    async fn get_topic_meta(
        &self,
        _workspace: &str,
        topic: &str,
    ) -> Result<Option<TopicFieldMeta>, EngineError> {
        Ok(self
            .store
            .topic(topic)
            .map(|t| topic_field_meta(t, &self.store.tokenized_for(topic))))
    }

    async fn get_topic_index_sizes(
        &self,
        _workspace: &str,
        _topics: &[String],
    ) -> Result<Vec<TopicIndexSize>, EngineError> {
        Ok(Vec::new())
    }

    async fn check_workspace_storage(
        &self,
        _workspace: &str,
    ) -> Result<WorkspaceStorageStatus, EngineError> {
        Ok(WorkspaceStorageStatus {
            schema_version: kaflow_api_types::CURRENT_SCHEMA_VERSION,
            current_schema_version: kaflow_api_types::CURRENT_SCHEMA_VERSION,
            topic_cf_count: self.store.topics.len(),
            has_legacy_data: false,
            requires_reset: false,
            reason: "mock engine — local fixture workspace".to_string(),
        })
    }

    async fn list_kafka_topics(&self, _bootstrap: &str) -> Result<Vec<String>, EngineError> {
        Ok(self.store.topics.iter().map(|t| t.name.clone()).collect())
    }

    async fn fetch_cluster_topology(
        &self,
        _bootstrap: &str,
        topics: &[String],
    ) -> Result<ClusterTopology, EngineError> {
        // 데모용 3-브로커 가상 클러스터. 파티션 leader 는 round-robin, replica=2(ISR=replica).
        const BROKERS: i32 = 3;
        let brokers = (1..=BROKERS)
            .map(|id| BrokerInfo {
                node_id: id,
                host: format!("mock-broker-{id}"),
                port: 9092 + id - 1,
                rack: Some(format!("rack-{}", (id - 1) % 2 + 1)),
                is_controller: id == 1,
            })
            .collect();

        // 요청 토픽(있으면)만, 없으면 전체 fixture 토픽.
        let wanted: Vec<&MockTopic> = if topics.is_empty() {
            self.store.topics.iter().collect()
        } else {
            topics.iter().filter_map(|n| self.store.topic(n)).collect()
        };
        let topic_topos = wanted
            .into_iter()
            .map(|t| TopicTopology {
                name: t.name.clone(),
                is_internal: false,
                partitions: (0..t.partition_count as i32)
                    .map(|p| {
                        let leader = p % BROKERS + 1;
                        let replica2 = leader % BROKERS + 1;
                        PartitionTopology {
                            partition_id: p,
                            leader_id: leader,
                            replica_ids: vec![leader, replica2],
                            isr_ids: vec![leader, replica2],
                        }
                    })
                    .collect(),
            })
            .collect();

        Ok(ClusterTopology {
            cluster_id: Some("mock-cluster-id".to_string()),
            controller_id: 1,
            brokers,
            topics: topic_topos,
        })
    }

    async fn get_cluster_id(&self, _bootstrap: &str) -> Result<String, EngineError> {
        Ok("mock-cluster-id".to_string())
    }

    async fn get_kafka_version_info(
        &self,
        _bootstrap: &str,
    ) -> Result<KafkaVersionInfo, EngineError> {
        Ok(KafkaVersionInfo {
            inferred_version: Some("≈ Kafka mock".to_string()),
            api_versions: vec![],
        })
    }

    async fn get_topic_message_count(
        &self,
        _bootstrap: &str,
        topic: &str,
    ) -> Result<TopicMessageCount, EngineError> {
        let (count, parts) = match self.store.topic(topic) {
            Some(t) => (t.messages.len() as i64, t.partition_count),
            None => (0, 1),
        };
        Ok(TopicMessageCount {
            topic: topic.to_string(),
            message_count: count,
            partition_count: parts,
        })
    }

    async fn list_topic_message_counts(
        &self,
        bootstrap: &str,
        topics: &[String],
    ) -> Result<Vec<TopicMessageCount>, EngineError> {
        let mut out = Vec::with_capacity(topics.len());
        for t in topics {
            out.push(self.get_topic_message_count(bootstrap, t).await?);
        }
        Ok(out)
    }

    async fn get_topic_size_profile(
        &self,
        _bootstrap: &str,
        topic: &str,
    ) -> Result<TopicSizeProfile, EngineError> {
        let t = self.store.topic(topic);
        let (avg_bytes, sampled) = match t {
            Some(t) if !t.messages.is_empty() => {
                let total: usize = t
                    .messages
                    .iter()
                    .map(|m| m.value_json.len() + m.key.as_ref().map(|k| k.len()).unwrap_or(0))
                    .sum();
                ((total / t.messages.len()) as u64, t.messages.len() as u32)
            }
            _ => (0, 0),
        };
        Ok(TopicSizeProfile {
            topic: topic.to_string(),
            avg_bytes,
            sampled,
            cleanup_policy: None,
            suggested_value_deserializer: t.map(|t| t.value_deserializer.clone()),
            suggested_reason: t.map(|_| "deserializer declared by mock fixture".to_string()),
            suggested_key_deserializer: t.map(|t| t.key_deserializer.clone()),
            suggested_key_reason: t.map(|_| "deserializer declared by mock fixture".to_string()),
        })
    }

    async fn suggest_tokenize_fields(
        &self,
        _bootstrap: &str,
        topic: &str,
        _key_spec: DeserializerSpec,
        _value_spec: DeserializerSpec,
    ) -> Result<SuggestTokenizeFieldsResponse, EngineError> {
        Ok(SuggestTokenizeFieldsResponse {
            topic: topic.to_string(),
            sampled: 0,
            candidates: Vec::new(),
        })
    }

    async fn save_workspace_cluster_id(
        &self,
        _workspace: &str,
        _cluster_id: &str,
    ) -> Result<(), EngineError> {
        Ok(())
    }

    async fn get_workspace_cluster_id(
        &self,
        _workspace: &str,
    ) -> Result<Option<String>, EngineError> {
        Ok(Some("mock-cluster-id".to_string()))
    }

    async fn get_topic_offset_status(
        &self,
        _workspace: &str,
        topic: &str,
        _bootstrap: &str,
    ) -> Result<TopicOffsetStatus, EngineError> {
        let t = match self.store.topic(topic) {
            Some(t) => t,
            None => {
                return Ok(TopicOffsetStatus {
                    topic: topic.to_string(),
                    kafka_total: 0,
                    indexed_total: 0,
                    processed_total: 0,
                    gap: 0,
                    skipped: 0,
                    skip_breakdown: Default::default(),
                    is_caught_up: true,
                    partitions: Vec::new(),
                })
            }
        };
        // partition → (min, max) indexed offset.
        let mut minmax: HashMap<u32, (i64, i64)> = HashMap::new();
        for m in &t.messages {
            let off = m.offset as i64;
            minmax
                .entry(m.partition)
                .and_modify(|(lo, hi)| {
                    *lo = (*lo).min(off);
                    *hi = (*hi).max(off);
                })
                .or_insert((off, off));
        }
        let mut partitions = Vec::new();
        let mut kafka_total = 0i64;
        for p in 0..t.partition_count {
            match minmax.get(&p) {
                Some((lo, hi)) => {
                    let latest = hi + 1; // high watermark
                    kafka_total += latest - *lo;
                    partitions.push(PartitionOffsetStatus {
                        partition: p as i32,
                        earliest: *lo,
                        latest,
                        min_indexed_offset: Some(*lo),
                        max_indexed_offset: Some(*hi),
                        gap: 0,
                    });
                }
                None => partitions.push(PartitionOffsetStatus {
                    partition: p as i32,
                    earliest: 0,
                    latest: 0,
                    min_indexed_offset: None,
                    max_indexed_offset: None,
                    gap: 0,
                }),
            }
        }
        let indexed_total = t.messages.len() as i64;
        Ok(TopicOffsetStatus {
            topic: topic.to_string(),
            kafka_total,
            indexed_total,
            // mock 은 전 구간 커버 (gap 0) — 커버 범위 = 서버 범위.
            processed_total: kafka_total,
            gap: 0,
            skipped: kafka_total - indexed_total,
            skip_breakdown: Default::default(),
            is_caught_up: true,
            partitions,
        })
    }

    async fn get_topic_config_info(
        &self,
        _workspace: &str,
        _bootstrap: &str,
        topic: &str,
    ) -> Result<TopicConfigInfoResponse, EngineError> {
        let parts = self
            .store
            .topic(topic)
            .map(|t| t.partition_count)
            .unwrap_or(1);
        Ok(TopicConfigInfoResponse {
            topic: topic.to_string(),
            cleanup_policy: "delete".to_string(),
            retention_ms: Some(-1),
            retention_bytes: None,
            compression_type: None,
            message_timestamp_type: None,
            partition_count: parts as usize,
            replication_factor: 1,
            topic_id: None,
            checked_at: 0,
        })
    }

    async fn list_watched_lag(
        &self,
        _workspace: &str,
        _bootstrap: &str,
        topics: Vec<String>,
    ) -> Result<Vec<TopicLagStatus>, EngineError> {
        Ok(topics
            .into_iter()
            .map(|topic| TopicLagStatus {
                topic,
                behind: 0,
                reachable: true,
            })
            .collect())
    }
}
