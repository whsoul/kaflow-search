# Read Kafka like a Database

Building a local index so a Kafka topic can be queried directly.

> **초안입니다.** 산문은 비워 두었습니다 — `TODO` 블록을 채우거나 지우면서 쓰시면 됩니다.
> 그림과 표, 수치는 채워져 있습니다. 발행 전에 이 인용 블록과 남은 `TODO` 를 모두 지워 주세요.

---

## Why this exists

> **TODO** — 문제부터. 로그 한 줄 찾자고 ELK 를 세워야 했던 상황을 짧게.
> 킥: *"난 단지 로그를 찾고 싶었을 뿐인데, ELK 를 다 구축해야만 하는가."*
> 여기는 3~4 문단이면 충분합니다. 제품 이야기는 아직 하지 않습니다.

---

## The stack you end up running

![As-is architecture](figures/as-is.svg)

To search messages you operate Kafka Connect (or Logstash) into Elasticsearch, a sink connector into a database, and ksqlDB or Flink for counting. Five components, none of which is the thing you wanted, plus three different clients to look at three different places.

> **TODO** — 실제로 운영해 본 입장에서 한 문단. 무엇이 제일 성가셨는지.

---

## What makes a local index possible

A Kafka topic is append-only, ordered within a partition, and addressable by offset. An index over it is therefore a pure function of `(topic, partition, offset range)` — it can be built incrementally, and resumed from wherever it stopped, without coordinating with anything else.

> **TODO** — 위 한 문단이 핵심 논거입니다. Kafka 개론으로 늘리지 마세요.
> 늘리면 "교육 포장을 씌운 홍보글" 로 읽힙니다.

---

## Doing it on the laptop instead

![To-be architecture](figures/to-be.svg)

Fetch by explicit partition and offset, build the index locally, search it locally. No consumer group is joined and no offsets are committed, so the cluster does not notice. Zero components to operate; one client.

> **TODO** — 인덱싱이 실제로 어떻게 도는지. key/value 파싱 → 항목 생성 → 저장 → 조회.
> 키 스키마 같은 내부는 개념 수준까지만.

---

## What it costs on disk

| Config | Messages | Entries / msg | Meta | Index | Total on disk |
| --- | ---: | ---: | ---: | ---: | ---: |
| FieldBased | 2,099,964 | 14.6 | 497 MB | 478 MB | **0.97 GB** |
| Full | 2,099,964 | 18.6 | 497 MB | 606 MB\* | **1.10 GB**\* |

A two-million-message topic costs about a gigabyte of local disk. On 100 GB of free space that is roughly 200 million messages, or a hundred topics of that size.

\* Projected, not measured: a full index of the same topic has not been captured yet. Derived from 18.6 entries per message and 15.5 bytes per entry.

**No comparison to the Kafka-side size is given.** The same messages occupy wildly different numbers of bytes on a broker depending on producer compression and batching, so it is not a stable baseline. What is reported here is what the local disk actually holds, after RocksDB's snappy compression.

Both rows come from one JSON topic. A different serialization format, or a topic with far more fields, will move these numbers.

---

## Not indexing everything

![Partial indexing](figures/partial-indexing.svg)

The index is trimmed along two axes — how far back, and how wide. Choosing which topics to index at all is the third and largest lever.

| Lever | Axis | Policy |
| --- | --- | --- |
| Index only the topics worth searching | topic | — |
| Keep the newest N per partition | messages | `CountBased { max_count }` |
| Keep only the fields you search | fields | `FieldBased` |
| Never trim this topic | — | `Pinned` |
| Drop this topic's index | topic | `DropIndex` |

Dropping fields removes them from *search*, not from the message — the body is still read in full.

Measured: `FieldBased` took entries from 18.6 to 14.6 per message on the topic above, about 21% off the index, or 128 MB.

> **TODO** — 어떤 필드를 버릴지 정하는 감각. 실제로 뭘 남기고 뭘 버렸는지 예시 하나.

---

## What you give up

| | Pipeline | Local index |
| --- | --- | --- |
| To stand it up | Connect, Elasticsearch, Kibana | install an app |
| Operating | always on; versions, shards, disk | none |
| Fixed cost | instances, every month | none — hardware you own |
| Where the data goes | replicated out of the cluster | stays on your disk |
| Scale ceiling | whatever you pay for | whatever the disk holds |
| **Sharing with a team** | everyone sees the same view | **no — each disk is its own** |
| Retention | keep it forever | Kafka retention plus local trimming |

The last two rows are the honest part. If a team needs one shared view, or the data has to outlive Kafka's retention, the pipeline is the right answer. A local index trades those two away to get everything else.

---

## Closing

> **TODO** — 마무리. 이 구조를 실제로 구현한 것이 Kaflow Search 라는 한 문단, 링크 하나.
> 기능 목록은 넣지 않습니다 — 그건 사이트가 합니다.
> *"만들었습니다"* 가 아니라 *"만들어 보니 위 숫자가 나왔다"* 의 톤으로.

---

## Before publishing

- [ ] `TODO` 블록과 이 절을 모두 삭제
- [ ] Full 행을 실측으로 교체 (같은 토픽 전체 인덱싱 → 진단 리포트)
- [ ] 다른 직렬화 형식 토픽 측정 한 건 추가 (선택)
- [ ] 마크다운을 `website/blog/<slug>.html` 로 옮기고 `figures/` 를 `website/` 아래로
- [ ] `sitemap.xml` 에 새 URL 추가
