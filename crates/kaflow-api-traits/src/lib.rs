//! Kaflow public API traits — public/private 경계의 인터페이스.
//!
//! 본 crate 는 **trait 정의 only**. 구현체는 다음 위치에 둔다:
//! - **mock 구현**: `crates/kaflow-mock-engine/` (미래)
//! - **실제 구현**: `private/crates/kaflow-engine-impl/` (미래, private repo 후보)
//!
//! 설계 문서: `docs/library_split_design.md` §3 (Engine trait), §5 (Deserializer / SchemaSource).
//!
//! 현재 상태: **trait skeleton 만 도입**. `KafkaToolEngine` 의 메서드는 `commands.rs` 의
//! Tauri command 를 점진적으로 흡수하면서 확장된다. 모든 메서드 시그니처는 `async-trait` 기반.

pub mod deserializer;
pub mod engine;
pub mod error;
pub mod progress;
pub mod schema;

// 평면 re-export — 호출처가 `kaflow_api_traits::KafkaToolEngine` 처럼 짧게 쓸 수 있게.
pub use deserializer::*;
pub use engine::*;
pub use error::*;
pub use progress::*;
pub use schema::*;
