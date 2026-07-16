//! 로컬 json fixture 기반 in-memory 데이터 소스 (mock-engine 의 "fake" 코어).
//!
//! 옛 sample mode(RocksDB 차용)의 public 후계 — Kafka 클러스터 / RocksDB 없이
//! **로컬 json 파일**을 source 로 동일한 인덱싱/조회 UX 를 재현한다.
//!
//! 설계 원칙(`.claude/CLAUDE.md` 합의):
//! - 실 엔진(private engine-impl)과 **동작 parity 추구 금지**. fixture 위 가장 단순하고
//!   정직한 동작(선형 스캔)까지만. 실 엔진 정확성은 engine-impl 테스트가 책임진다.
//! - 인프라 의존 0 — `std::fs`/`std::env`/`serde_json` + 경량 `kaflow-tokenizer`(deps 0) 만
//!   사용(tokio/tauri/rocksdb/kafka X).
//!
//! 필드 표기는 실 엔진 `flatten_json` 과 동일: `P.orderId` / `P.items[*].name` / `K.id`.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::RwLock;

use kaflow_tokenizer::{Tokenizer, WhitespaceTokenizer};

use kaflow_api_types::{
    LocItem, MessageResult, OffsetBucket, OffsetBucketsDeliveryMode, OffsetBucketsResponse,
    PartitionOffsetBuckets, PosFilter, PrefixLeaf, QueryNode, RegistryResource, SearchMode,
    SortOrder, TimeBucketDeliveryMode, TimeBucketRow, TimeBucketsResponse, TsRange,
};

// ── 모델 ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct MockMessage {
    pub partition: u32,
    pub offset: u64,
    pub ts_millis: u64,
    /// 디코드된 key 텍스트 (raw). tombstone/없음 = None.
    pub key: Option<String>,
    /// value 의 pretty json 문자열 (본문 표시용).
    pub value_json: String,
    /// flatten 된 `K.*`/`P.*` path → leaf value 목록 (인덱스 entry 의 출처).
    pub fields: Vec<(String, String)>,
}

#[derive(Debug, Clone)]
pub struct MockTopic {
    pub name: String,
    pub partition_count: u32,
    pub key_deserializer: String,
    pub value_deserializer: String,
    pub key_fields: Vec<String>,
    pub payload_fields: Vec<String>,
    /// (partition asc, offset asc) 정렬.
    pub messages: Vec<MockMessage>,
}

#[derive(Debug, Default)]
pub struct MockStore {
    pub topics: Vec<MockTopic>,
    /// 런타임 어절(tokenize) 상태 — topic → 어절 대상 필드명 집합. mutation API
    /// (`set_indexed_fields` / `set_tokenize_fields`)로 갱신, 검색이 라이브로 읽는다.
    /// 옛 sample mode 와 달리 데모용 in-memory 상태(RwLock, &self 메서드라 내부 가변).
    tokenized: RwLock<HashMap<String, BTreeSet<String>>>,
    /// 런타임 Schema Registry 리소스 목록 — id → 리소스. `RegistryApi` CRUD 가 갱신.
    /// **세션 동안만 유지**(재기동 시 사라짐). 실 엔진은 `~/.kaflow/schema_registries.json` 영속.
    /// 연결 테스트/스키마 조회는 네트워크라 mock 은 stub — 등록/목록/삭제만 실제로 동작한다.
    registries: RwLock<Vec<RegistryResource>>,
}

// ── fixture 직렬화 포맷 ───────────────────────────────────────────────────────

#[derive(serde::Deserialize)]
struct FxRoot {
    #[serde(default)]
    topics: Vec<FxTopic>,
}

#[derive(serde::Deserialize)]
struct FxTopic {
    name: String,
    #[serde(default)]
    partitions: Option<u32>,
    #[serde(default = "default_deser")]
    key_deserializer: String,
    #[serde(default = "default_deser")]
    value_deserializer: String,
    #[serde(default)]
    messages: Vec<FxMessage>,
}

#[derive(serde::Deserialize)]
struct FxMessage {
    #[serde(default)]
    partition: u32,
    #[serde(default)]
    offset: Option<u64>,
    #[serde(default)]
    ts_millis: Option<u64>,
    #[serde(default)]
    key: Option<serde_json::Value>,
    #[serde(default)]
    value: serde_json::Value,
}

fn default_deser() -> String {
    "json".to_string()
}

// ── 로딩 ────────────────────────────────────────────────────────────────────

impl MockStore {
    /// 환경변수 `KAFLOW_MOCK_FIXTURES`(파일 경로) 가 있으면 그 파일을, 없으면 번들된
    /// `fixtures/default.json` 을 읽어 적재한다. 파싱 실패 시 빈 store(데모 없음).
    pub fn load() -> Self {
        let raw = load_fixture_text();
        match serde_json::from_str::<FxRoot>(&raw) {
            Ok(root) => Self::from_fixture(root),
            Err(_) => Self::default(),
        }
    }

    fn from_fixture(root: FxRoot) -> Self {
        let topics = root.topics.into_iter().map(build_topic).collect();
        Self {
            topics,
            tokenized: RwLock::new(HashMap::new()),
            registries: RwLock::new(Vec::new()),
        }
    }

    pub fn topic(&self, name: &str) -> Option<&MockTopic> {
        self.topics.iter().find(|t| t.name == name)
    }

    // ── Schema Registry 리소스 CRUD (in-memory, 세션 한정) ──────────────────
    /// 등록된 리소스 목록 (name 오름차순 — 실 엔진과 동일 정렬).
    pub fn list_registries(&self) -> Vec<RegistryResource> {
        let mut v = self.registries.read().unwrap().clone();
        v.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        v
    }

    /// id 기준 upsert.
    pub fn save_registry(&self, resource: RegistryResource) {
        let mut guard = self.registries.write().unwrap();
        if let Some(existing) = guard.iter_mut().find(|r| r.id == resource.id) {
            *existing = resource;
        } else {
            guard.push(resource);
        }
    }

    /// id 기준 삭제.
    pub fn delete_registry(&self, id: &str) {
        self.registries.write().unwrap().retain(|r| r.id != id);
    }

    /// `filter` = None → 전체, Some → 그 이름들만(존재하는 것).
    pub fn topics_for(&self, filter: Option<&[String]>) -> Vec<&MockTopic> {
        match filter {
            None => self.topics.iter().collect(),
            Some(names) => names.iter().filter_map(|n| self.topic(n)).collect(),
        }
    }

    /// 어절 대상 필드 교체(빈 목록 = 해제). `set_indexed_fields`/`set_tokenize_fields` 가 호출.
    pub fn set_tokenized(&self, topic: &str, fields: Vec<String>) {
        let mut g = self.tokenized.write().unwrap();
        let set: BTreeSet<String> = fields.into_iter().collect();
        if set.is_empty() {
            g.remove(topic);
        } else {
            g.insert(topic.to_string(), set);
        }
    }

    /// 토픽의 현재 어절 대상 필드 집합 (없으면 빈 set).
    pub fn tokenized_for(&self, topic: &str) -> BTreeSet<String> {
        self.tokenized
            .read()
            .unwrap()
            .get(topic)
            .cloned()
            .unwrap_or_default()
    }
}

fn load_fixture_text() -> String {
    if let Ok(path) = std::env::var("KAFLOW_MOCK_FIXTURES") {
        if let Ok(txt) = std::fs::read_to_string(&path) {
            return txt;
        }
    }
    include_str!("../fixtures/default.json").to_string()
}

fn build_topic(fx: FxTopic) -> MockTopic {
    // partition 별 auto-offset 카운터 (offset 미지정 시).
    let mut next_off: BTreeMap<u32, u64> = BTreeMap::new();
    // ts 미지정 시 합성용 base (2024-01-01) + 메시지마다 1분 증가.
    let base_ts: u64 = 1_704_067_200_000;
    let mut messages: Vec<MockMessage> = Vec::with_capacity(fx.messages.len());

    for (i, m) in fx.messages.into_iter().enumerate() {
        let partition = m.partition;
        let offset = match m.offset {
            Some(o) => {
                next_off.insert(partition, o + 1);
                o
            }
            None => {
                let slot = next_off.entry(partition).or_insert(0);
                let o = *slot;
                *slot += 1;
                o
            }
        };
        let ts_millis = m.ts_millis.unwrap_or(base_ts + (i as u64) * 60_000);

        let mut fields: Vec<(String, String)> = Vec::new();
        if let Some(k) = &m.key {
            flatten_json("K", k, &mut fields);
        }
        flatten_json("P", &m.value, &mut fields);

        let key = m.key.as_ref().map(scalar_or_compact);
        let value_json =
            serde_json::to_string_pretty(&m.value).unwrap_or_else(|_| m.value.to_string());

        messages.push(MockMessage {
            partition,
            offset,
            ts_millis,
            key,
            value_json,
            fields,
        });
    }

    messages.sort_by(|a, b| a.partition.cmp(&b.partition).then(a.offset.cmp(&b.offset)));

    let key_fields = unique_paths(&messages, "K.");
    let payload_fields = unique_paths(&messages, "P.");
    let partition_count = fx
        .partitions
        .unwrap_or_else(|| messages.iter().map(|m| m.partition + 1).max().unwrap_or(1))
        .max(1);

    MockTopic {
        name: fx.name,
        partition_count,
        key_deserializer: fx.key_deserializer,
        value_deserializer: fx.value_deserializer,
        key_fields,
        payload_fields,
        messages,
    }
}

/// 객체 key 면 compact json, 스칼라면 그 값 텍스트.
fn scalar_or_compact(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Null => String::new(),
        other => other.to_string(),
    }
}

fn unique_paths(messages: &[MockMessage], prefix: &str) -> Vec<String> {
    let mut seen: Vec<String> = Vec::new();
    for m in messages {
        for (p, _) in &m.fields {
            if (p.starts_with(prefix) || p == prefix.trim_end_matches('.')) && !seen.contains(p) {
                seen.push(p.clone());
            }
        }
    }
    seen
}

/// 실 엔진 `message_parse::flatten_json` 과 동일 규칙(배열 = `prefix[*]`).
pub fn flatten_json(prefix: &str, value: &serde_json::Value, out: &mut Vec<(String, String)>) {
    match value {
        serde_json::Value::Object(map) => {
            for (k, v) in map {
                flatten_json(&format!("{prefix}.{k}"), v, out);
            }
        }
        serde_json::Value::Array(arr) => {
            let path = format!("{prefix}[*]");
            for v in arr {
                flatten_json(&path, v, out);
            }
        }
        serde_json::Value::String(s) => out.push((prefix.to_string(), s.clone())),
        serde_json::Value::Number(n) => out.push((prefix.to_string(), n.to_string())),
        serde_json::Value::Bool(b) => out.push((prefix.to_string(), b.to_string())),
        serde_json::Value::Null => {}
    }
}

// ── 매칭 술어 ─────────────────────────────────────────────────────────────────

fn ts_ok(m: &MockMessage, ts: Option<&TsRange>) -> bool {
    match ts {
        None => true,
        Some(r) => {
            r.gte_ms.map(|lo| m.ts_millis >= lo).unwrap_or(true)
                && r.lte_ms.map(|hi| m.ts_millis <= hi).unwrap_or(true)
        }
    }
}

fn pos_ok(m: &MockMessage, pos: Option<&PosFilter>) -> bool {
    match pos {
        None => true,
        Some(p) => p.passes(m.partition, m.offset),
    }
}

fn field_selected(path: &str, fields: Option<&[String]>) -> bool {
    match fields {
        None => true,
        Some(fs) => fs.iter().any(|f| f == path),
    }
}

/// 값 매칭. `tokenized=true` 면 값을 **어절**(공백 분해 + 무사전 CJK 글자 분해)로 보고 단어
/// 단위로 매칭한다(문장 중간 단어 검색 = 어절 데모). 단어 prefix 매칭 = 실 엔진 "단어 앞부분".
/// 공통 [`kaflow_tokenizer::WhitespaceTokenizer`] 를 써서 일/중도 분절된다(실엔진은 charabia).
fn value_matches(val: &str, query: &str, mode: &SearchMode, tokenized: bool) -> bool {
    let hit = |s: &str| match mode {
        SearchMode::Exact => s == query,
        SearchMode::Prefix => s.starts_with(query),
    };
    if !tokenized {
        return hit(val);
    }
    let val_toks = WhitespaceTokenizer.tokenize(val);
    let q_toks = WhitespaceTokenizer.tokenize(query);
    if q_toks.len() <= 1 {
        // 단일 토큰 질의(한/영 어절, 단일 CJK 글자) — query 통째로 어절 prefix/exact 매칭.
        val_toks.iter().any(|w| hit(w))
    } else {
        // 다토큰 질의(공백 없는 CJK 다글자 등) — 모든 질의 토큰이 어떤 val 토큰과 매칭.
        q_toks.iter().all(|qt| {
            val_toks.iter().any(|w| match mode {
                SearchMode::Exact => w == qt,
                SearchMode::Prefix => w.starts_with(qt.as_str()),
            })
        })
    }
}

// ── multi (QueryNode) 평가 ────────────────────────────────────────────────────

fn field_match(path: &str, field: &Option<String>) -> bool {
    match field {
        None => true,
        Some(f) => path == f,
    }
}

fn eval_node(m: &MockMessage, node: &QueryNode, tok: &BTreeSet<String>) -> bool {
    match node {
        QueryNode::Term(t) => m.fields.iter().any(|(p, v)| {
            field_match(p, &t.field)
                && value_matches(v, &t.value, &SearchMode::Exact, tok.contains(p))
        }),
        QueryNode::Prefix(PrefixLeaf { field, value }) => m.fields.iter().any(|(p, v)| {
            field_match(p, field) && value_matches(v, value, &SearchMode::Prefix, tok.contains(p))
        }),
        QueryNode::Exists(e) => m.fields.iter().any(|(p, _)| p == &e.field),
        QueryNode::Range(r) => m.fields.iter().any(|(p, v)| {
            p == &r.field
                && r.gte
                    .as_ref()
                    .map(|lo| v.as_str() >= lo.as_str())
                    .unwrap_or(true)
                && r.lte
                    .as_ref()
                    .map(|hi| v.as_str() <= hi.as_str())
                    .unwrap_or(true)
        }),
        QueryNode::Bool(b) => {
            let must_ok = b.must.iter().all(|n| eval_node(m, n, tok));
            let must_not_ok = b.must_not.iter().all(|n| !eval_node(m, n, tok));
            let should_ok = if b.should.is_empty() {
                true
            } else {
                let min = b
                    .minimum_should_match
                    .unwrap_or(if b.must.is_empty() { 1 } else { 0 });
                b.should.iter().filter(|n| eval_node(m, n, tok)).count() as u32 >= min
            };
            must_ok && must_not_ok && should_ok
        }
    }
}

// ── loc / 정렬 ────────────────────────────────────────────────────────────────

pub(crate) fn sort_locs(v: &mut [LocItem], sort: &SortOrder) {
    match sort {
        SortOrder::NewestFirst => v.sort_by(|a, b| {
            b.ts_millis
                .cmp(&a.ts_millis)
                .then(b.partition.cmp(&a.partition))
                .then(b.offset.cmp(&a.offset))
        }),
        SortOrder::OldestFirst => v.sort_by(|a, b| {
            a.ts_millis
                .cmp(&b.ts_millis)
                .then(a.partition.cmp(&b.partition))
                .then(a.offset.cmp(&b.offset))
        }),
    }
}

fn browse_loc(m: &MockMessage) -> LocItem {
    LocItem {
        ts_millis: m.ts_millis,
        partition: m.partition,
        offset: m.offset,
        matched_field: None,
        pretty_index_key: None,
        raw_index_key_hex: None,
    }
}

fn hit_loc(topic: &str, m: &MockMessage, field: &str, term: &str) -> LocItem {
    let pretty = fmt_index_key(topic, field, term, m.partition, m.offset);
    LocItem {
        ts_millis: m.ts_millis,
        partition: m.partition,
        offset: m.offset,
        matched_field: Some(field.to_string()),
        raw_index_key_hex: Some(to_hex(pretty.as_bytes())),
        pretty_index_key: Some(pretty),
    }
}

fn fmt_index_key(topic: &str, field: &str, term: &str, partition: u32, offset: u64) -> String {
    format!("I | {topic} | {field} | {term} | p{partition} | o{offset}")
}

fn fmt_meta_key(topic: &str, partition: u32, offset: u64) -> String {
    format!("M | {topic} | p{partition} | o{offset}")
}

fn to_hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

// ── 조회 API (MockTopic 위 thin 어댑터의 공유 코어) ───────────────────────────

impl MockTopic {
    /// browse — 전체 메시지(쿼리 없음), ts/pos 필터 + 정렬 + limit.
    pub fn browse_locs(
        &self,
        ts: Option<&TsRange>,
        pos: Option<&PosFilter>,
        sort: &SortOrder,
        limit: Option<usize>,
    ) -> (Vec<LocItem>, usize) {
        let mut v: Vec<LocItem> = self
            .messages
            .iter()
            .filter(|m| ts_ok(m, ts) && pos_ok(m, pos))
            .map(browse_loc)
            .collect();
        sort_locs(&mut v, sort);
        let total = v.len();
        if let Some(l) = limit {
            v.truncate(l);
        }
        (v, total)
    }

    /// keyword — 어떤 (선택된) 필드 값이 query 와 매치하면 hit. matched_field = 첫 매치 필드.
    /// 반환: (locs, hit_fields union, total).
    #[allow(clippy::too_many_arguments)]
    pub fn keyword_locs(
        &self,
        query: &str,
        fields: Option<&[String]>,
        mode: &SearchMode,
        tok: &BTreeSet<String>,
        ts: Option<&TsRange>,
        pos: Option<&PosFilter>,
        sort: &SortOrder,
        limit: Option<usize>,
    ) -> (Vec<LocItem>, Vec<String>, usize) {
        let mut locs: Vec<LocItem> = Vec::new();
        let mut hit_fields: Vec<String> = Vec::new();
        if query.is_empty() {
            return (locs, hit_fields, 0);
        }
        for m in &self.messages {
            if !ts_ok(m, ts) || !pos_ok(m, pos) {
                continue;
            }
            if let Some((path, value)) = m.fields.iter().find(|(p, v)| {
                field_selected(p, fields) && value_matches(v, query, mode, tok.contains(p))
            }) {
                if !hit_fields.contains(path) {
                    hit_fields.push(path.clone());
                }
                // 어절 필드면 매치된 단어를 인덱스키 term 으로(본문 전체 대신 깔끔하게).
                let term: String = if tok.contains(path) {
                    value
                        .split_whitespace()
                        .find(|w| match mode {
                            SearchMode::Exact => *w == query,
                            SearchMode::Prefix => w.starts_with(query),
                        })
                        .unwrap_or(value)
                        .to_string()
                } else {
                    value.clone()
                };
                locs.push(hit_loc(&self.name, m, path, &term));
            }
        }
        sort_locs(&mut locs, sort);
        let total = locs.len();
        if let Some(l) = limit {
            locs.truncate(l);
        }
        (locs, hit_fields, total)
    }

    /// multi — QueryNode 트리 평가. matched_field 는 표시용으로 비움(트리 결과라 단일 필드 의미 약함).
    pub fn multi_locs(
        &self,
        node: &QueryNode,
        tok: &BTreeSet<String>,
        ts: Option<&TsRange>,
        pos: Option<&PosFilter>,
        sort: &SortOrder,
        limit: Option<usize>,
    ) -> (Vec<LocItem>, usize) {
        let mut v: Vec<LocItem> = self
            .messages
            .iter()
            .filter(|m| ts_ok(m, ts) && pos_ok(m, pos) && eval_node(m, node, tok))
            .map(browse_loc)
            .collect();
        sort_locs(&mut v, sort);
        let total = v.len();
        if let Some(l) = limit {
            v.truncate(l);
        }
        (v, total)
    }

    /// loc 목록 → MessageResult (본문 fetch). 없는 loc 은 건너뜀.
    pub fn message_rows(&self, locs: &[LocItem]) -> Vec<MessageResult> {
        let mut index: BTreeMap<(u32, u64), &MockMessage> = BTreeMap::new();
        for m in &self.messages {
            index.insert((m.partition, m.offset), m);
        }
        locs.iter()
            .filter_map(|loc| {
                let m = index.get(&(loc.partition, loc.offset))?;
                Some(self.message_result(
                    m,
                    loc.matched_field.clone(),
                    loc.pretty_index_key.clone(),
                ))
            })
            .collect()
    }

    fn message_result(
        &self,
        m: &MockMessage,
        matched_field: Option<String>,
        pretty_index_key: Option<String>,
    ) -> MessageResult {
        let meta_index_entries: Vec<(String, String)> = m
            .fields
            .iter()
            .map(|(p, v)| {
                (
                    p.clone(),
                    fmt_index_key(&self.name, p, v, m.partition, m.offset),
                )
            })
            .collect();
        MessageResult {
            matched_field,
            raw_index_key_hex: pretty_index_key.as_ref().map(|k| to_hex(k.as_bytes())),
            pretty_index_key,
            raw_meta_key_hex: to_hex(fmt_meta_key(&self.name, m.partition, m.offset).as_bytes()),
            pretty_meta_key: fmt_meta_key(&self.name, m.partition, m.offset),
            content_source: "full".to_string(),
            meta_index_entries,
            raw_meta_value_json: Some(m.value_json.clone()),
            field_values: m.fields.clone(),
            topic: self.name.clone(),
            partition: m.partition as i32,
            offset: m.offset as i64,
            timestamp: fmt_ts(m.ts_millis),
            key: m.key.clone().unwrap_or_default(),
            payload: m.value_json.clone(),
            headers: Vec::new(),
            json: m.value_json.clone(),
        }
    }
}

// ── bucket 빌더 (locs → 집계) ─────────────────────────────────────────────────

/// time buckets — `(partition, floor(ts/gran)*gran)` 별 count + offsets inline.
pub fn build_time_buckets(locs: &[LocItem], gran_ms: u64) -> TimeBucketsResponse {
    let gran = gran_ms.max(1);
    // (partition, gran_start) → offsets
    let mut groups: BTreeMap<(u32, u64), Vec<u64>> = BTreeMap::new();
    for loc in locs {
        let start = (loc.ts_millis / gran) * gran;
        groups
            .entry((loc.partition, start))
            .or_default()
            .push(loc.offset);
    }
    let total = locs.len() as u64;
    let rows = groups
        .into_iter()
        .map(|((partition, gran_start_ms), mut offsets)| {
            offsets.sort_unstable();
            TimeBucketRow {
                partition,
                gran_start_ms,
                count: offsets.len() as u64,
                offsets: Some(offsets),
            }
        })
        .collect();
    TimeBucketsResponse {
        gran_ms: gran,
        total,
        rows,
        delivery_mode: TimeBucketDeliveryMode::OffsetsInline,
        chosen_plan: Some("mock.time_buckets".to_string()),
    }
}

/// offset buckets — partition 별 `bucket_unit(=100)` 단위 leaf + offsets inline.
pub fn build_offset_buckets(topic: &str, locs: &[LocItem]) -> OffsetBucketsResponse {
    const UNIT: u64 = 100;
    // partition → (bucket_from → offsets)
    let mut by_part: BTreeMap<u32, BTreeMap<u64, Vec<u64>>> = BTreeMap::new();
    for loc in locs {
        let from = (loc.offset / UNIT) * UNIT;
        by_part
            .entry(loc.partition)
            .or_default()
            .entry(from)
            .or_default()
            .push(loc.offset);
    }
    let total = locs.len() as u64;
    let partitions = by_part
        .into_iter()
        .map(|(partition, buckets_map)| {
            let mut count: u64 = 0;
            let mut min_offset = u64::MAX;
            let mut max_offset = 0u64;
            let buckets: Vec<OffsetBucket> = buckets_map
                .into_iter()
                .map(|(from, mut offsets)| {
                    offsets.sort_unstable();
                    count += offsets.len() as u64;
                    min_offset = min_offset.min(*offsets.first().unwrap());
                    max_offset = max_offset.max(*offsets.last().unwrap());
                    OffsetBucket {
                        from_offset: from,
                        to_offset: from + UNIT - 1,
                        count: offsets.len() as u32,
                        offsets: Some(offsets),
                    }
                })
                .collect();
            if min_offset == u64::MAX {
                min_offset = 0;
            }
            PartitionOffsetBuckets {
                partition,
                count,
                min_offset,
                max_offset,
                bucket_unit: UNIT as u32,
                buckets,
            }
        })
        .collect();
    OffsetBucketsResponse {
        topic: topic.to_string(),
        total,
        partitions,
        delivery_mode: OffsetBucketsDeliveryMode::OffsetsInline,
        chosen_plan: Some("mock.offset_buckets".to_string()),
    }
}

// ── ts 포맷 (epoch ms → ISO8601 UTC, chrono 없이) ─────────────────────────────

fn fmt_ts(ms: u64) -> String {
    let secs = (ms / 1000) as i64;
    let days = secs.div_euclid(86_400);
    let rem = secs.rem_euclid(86_400);
    let (h, mi, s) = (rem / 3600, (rem % 3600) / 60, rem % 60);
    // Howard Hinnant civil_from_days.
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if month <= 2 { y + 1 } else { y };
    format!("{year:04}-{month:02}-{d:02}T{h:02}:{mi:02}:{s:02}Z")
}

#[cfg(test)]
mod tests {
    use super::*;
    use kaflow_api_types::{BoolNode, TermLeaf};

    fn store() -> MockStore {
        MockStore::load()
    }

    #[test]
    fn fixture_loads_topics_and_fields() {
        let s = store();
        assert!(!s.topics.is_empty(), "default fixture must have topics");
        let t = &s.topics[0];
        assert!(!t.messages.is_empty());
        assert!(
            t.payload_fields
                .iter()
                .all(|f| f.starts_with("P.") || f == "P"),
            "payload fields must use the P notation: {:?}",
            t.payload_fields
        );
    }

    #[test]
    fn flatten_uses_star_for_arrays() {
        let mut out = Vec::new();
        flatten_json(
            "P",
            &serde_json::json!({"items": [{"name": "a"}, {"name": "b"}]}),
            &mut out,
        );
        assert!(out.contains(&("P.items[*].name".to_string(), "a".to_string())));
        assert!(out.contains(&("P.items[*].name".to_string(), "b".to_string())));
    }

    #[test]
    fn keyword_prefix_matches_and_collects_hit_fields() {
        let s = store();
        let t = &s.topics[0];
        // 첫 메시지의 첫 필드 값을 prefix 로 검색하면 최소 1건 매치.
        let (path, val) = t.messages[0].fields[0].clone();
        let prefix: String = val.chars().take(2).collect();
        let (locs, hits, total) = t.keyword_locs(
            &prefix,
            None,
            &SearchMode::Prefix,
            &BTreeSet::new(),
            None,
            None,
            &SortOrder::NewestFirst,
            None,
        );
        assert!(total >= 1);
        assert!(!locs.is_empty());
        assert!(hits.contains(&path) || !hits.is_empty());
    }

    #[test]
    fn tokenized_field_matches_mid_sentence_word() {
        // 어절 미적용: 문장 중간 단어는 prefix(앞부분)로 안 잡힘.
        let s = store();
        let t = s.topic("reviews-ko").expect("reviews-ko fixture");
        // body 의 두 번째 어절(중간 단어)을 고른다.
        let body = t.messages[0]
            .fields
            .iter()
            .find(|(p, _)| p == "P.body")
            .map(|(_, v)| v.clone())
            .expect("P.body");
        let words: Vec<&str> = body.split_whitespace().collect();
        assert!(words.len() >= 3, "body must be multiple words: {body}");
        let mid = words[2];
        let fields = vec!["P.body".to_string()];

        let plain = t.keyword_locs(
            mid,
            Some(&fields),
            &SearchMode::Prefix,
            &BTreeSet::new(),
            None,
            None,
            &SortOrder::NewestFirst,
            None,
        );
        assert_eq!(plain.2, 0, "without tokenize, middle words must not match");

        let mut tok = BTreeSet::new();
        tok.insert("P.body".to_string());
        let toked = t.keyword_locs(
            mid,
            Some(&fields),
            &SearchMode::Prefix,
            &tok,
            None,
            None,
            &SortOrder::NewestFirst,
            None,
        );
        assert!(toked.2 >= 1, "with tokenize, middle words must match");
    }

    #[test]
    fn multi_term_eval() {
        let s = store();
        let t = &s.topics[0];
        let (field, value) = t.messages[0].fields[0].clone();
        let node = QueryNode::Bool(BoolNode {
            must: vec![QueryNode::Term(TermLeaf {
                field: Some(field),
                value,
            })],
            ..Default::default()
        });
        let (locs, total) = t.multi_locs(
            &node,
            &BTreeSet::new(),
            None,
            None,
            &SortOrder::NewestFirst,
            None,
        );
        assert!(total >= 1);
        assert!(!locs.is_empty());
    }

    #[test]
    fn buckets_count_matches_locs() {
        let s = store();
        let t = &s.topics[0];
        let (locs, total) = t.browse_locs(None, None, &SortOrder::OldestFirst, None);
        let tb = build_time_buckets(&locs, 60_000);
        assert_eq!(tb.total as usize, total);
        let ob = build_offset_buckets(&t.name, &locs);
        assert_eq!(ob.total as usize, total);
    }
}
