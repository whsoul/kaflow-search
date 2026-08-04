//! Kafka 인증 / 보안 프로토콜 DTO.
//!
//! 본 모듈은 enum 정의와 변환 헬퍼만 담당. 실제 handshake / TCP I/O 는 `kafka-client` (private)
//! 가 별도 구현. `AuthState` (runtime 메모리 캐시) 도 private 측에 있다.
//!
//! 저장 정책:
//! - `KafkaAuth` (런타임용): `password` 등 비밀 정보 포함. Tauri command 인자 / 내부 호출용.
//! - `StoredAuthConfig` (clusters.json 저장용): protocol / sasl_mechanism / username / cert 경로
//!   등 비-비밀 정보만 저장. password 는 매 연결 시 화면에서 입력.

use serde::{Deserialize, Serialize};

/// 보안 프로토콜 식별자. UI 표시 / 로깅 / `StoredAuthConfig` 키로 사용.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KafkaProtocol {
    #[serde(rename = "PLAINTEXT")]
    Plaintext,
    #[serde(rename = "SASL_PLAINTEXT")]
    SaslPlaintext,
    #[serde(rename = "SSL")]
    Ssl,
    #[serde(rename = "SASL_SSL")]
    SaslSsl,
}

impl Default for KafkaProtocol {
    fn default() -> Self {
        KafkaProtocol::Plaintext
    }
}

/// SASL 인증 메커니즘.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SaslMechanism {
    #[serde(rename = "PLAIN")]
    Plain,
    #[serde(rename = "SCRAM-SHA-256")]
    ScramSha256,
    #[serde(rename = "SCRAM-SHA-512")]
    ScramSha512,
    /// 표준 OAUTHBEARER (RFC 7628) — Bearer 토큰 직접 입력.
    /// 토큰은 `password` 필드 재사용 (username 미사용).
    #[serde(rename = "OAUTHBEARER")]
    OAuthBearer,
    /// AWS MSK IAM — 와이어는 OAUTHBEARER, 토큰은 SigV4 signer 가 연결 시마다 생성 (§5-F).
    /// username=Access Key ID / password=Secret Access Key + `aws_region`(필수) /
    /// `aws_session_token`(옵션, SaslSsl 필드). TLS 필수 → SASL_SSL 전용.
    #[serde(rename = "AWS_MSK_IAM")]
    AwsMskIam,
}

impl SaslMechanism {
    /// Kafka SaslHandshake/Authenticate 요청에 들어가는 와이어 식별자.
    pub fn wire_name(&self) -> &'static str {
        match self {
            SaslMechanism::Plain => "PLAIN",
            SaslMechanism::ScramSha256 => "SCRAM-SHA-256",
            SaslMechanism::ScramSha512 => "SCRAM-SHA-512",
            SaslMechanism::OAuthBearer => "OAUTHBEARER",
            SaslMechanism::AwsMskIam => "OAUTHBEARER",
        }
    }
}

/// 런타임 인증 설정. password / key_password 등 비밀 정보 포함.
///
/// FE → BE Tauri 커맨드 인자, 그리고 BE 내부 kafka 호출에 사용한다.
/// 저장 시에는 `StoredAuthConfig` 로 변환하여 비밀 정보를 제거한다.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "protocol")]
pub enum KafkaAuth {
    #[serde(rename = "PLAINTEXT")]
    None,
    #[serde(rename = "SASL_PLAINTEXT")]
    SaslPlaintext {
        mechanism: SaslMechanism,
        username: String,
        password: String,
    },
    #[serde(rename = "SSL")]
    Ssl {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        ca_cert_path: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        client_cert_path: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        client_key_path: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        key_password: Option<String>,
    },
    #[serde(rename = "SASL_SSL")]
    SaslSsl {
        mechanism: SaslMechanism,
        username: String,
        password: String,
        /// AWS MSK IAM 전용 — SigV4 서명 region (예: "ap-northeast-2"). 그 외 메커니즘 무시.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        aws_region: Option<String>,
        /// AWS MSK IAM 전용 — 임시 자격증명(STS/SSO) 사용 시 session token.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        aws_session_token: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        ca_cert_path: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        client_cert_path: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        client_key_path: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        key_password: Option<String>,
    },
}

impl Default for KafkaAuth {
    fn default() -> Self {
        KafkaAuth::None
    }
}

impl KafkaAuth {
    pub fn protocol(&self) -> KafkaProtocol {
        match self {
            KafkaAuth::None => KafkaProtocol::Plaintext,
            KafkaAuth::SaslPlaintext { .. } => KafkaProtocol::SaslPlaintext,
            KafkaAuth::Ssl { .. } => KafkaProtocol::Ssl,
            KafkaAuth::SaslSsl { .. } => KafkaProtocol::SaslSsl,
        }
    }

    /// 미구현/불가 protocol×mechanism 조합인 경우 표준 에러 메시지 반환.
    /// 호출부에서 `if let Some(e) = auth.unsupported_error() { return Err(e); }` 패턴으로 사용.
    ///
    /// 유일한 차단 조합 = SASL_PLAINTEXT × AWS_MSK_IAM (MSK 는 TLS 강제 — 평문 조합이
    /// 성립하지 않음). 나머지 전 조합 지원 (2026-07-06).
    pub fn unsupported_error(&self) -> Option<String> {
        match self {
            KafkaAuth::SaslPlaintext {
                mechanism: SaslMechanism::AwsMskIam,
                ..
            } => Some(
                "AWS MSK IAM requires TLS — select SASL_SSL as the security protocol".to_string(),
            ),
            KafkaAuth::None
            | KafkaAuth::Ssl { .. }
            | KafkaAuth::SaslPlaintext { .. }
            | KafkaAuth::SaslSsl { .. } => None,
        }
    }
}

/// AWS 자격증명 파일(`~/.aws/credentials`)의 프로파일 **요약** — 비밀 제외.
///
/// 목록 표시 전용이라 secret 은 담지 않는다. 사용자가 하나를 고른 뒤에만
/// `AwsProfileCredentials` 로 그 프로파일의 비밀을 읽는다 (전체 secret 을 한꺼번에
/// 화면으로 보내지 않기 위함).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AwsProfileSummary {
    pub name: String,
    /// 액세스 키 ID — 비밀이 아니라 식별자라 표시용으로 내려준다. SSO 전용이면 None.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub access_key_id: Option<String>,
    /// `~/.aws/config` 또는 credentials 파일에서 찾은 region (있으면 폼에 자동 반영).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
    /// 임시 자격증명(STS/SSO/aws-vault 등) — session token 을 함께 쓴다.
    pub has_session_token: bool,
    /// 액세스 키 없이 SSO/역할만 정의된 프로파일 — 이 앱에서 바로 쓸 수 없다.
    /// (`aws-auth-coverage.md` 항목 2 = SSO 자동 갱신, 엔터프라이즈 트랙.)
    pub sso_only: bool,
}

/// 엔진이 **실제로 보는** AWS 파일 경로. 화면이 "어디서 읽었는지" 를 정직하게 표시하기 위한 것.
///
/// FE 가 `~/.aws/...` 를 하드코딩해 보여주면 `AWS_SHARED_CREDENTIALS_FILE` / `AWS_CONFIG_FILE`
/// 이 설정된 환경에서 **거짓 경로**를 찍게 된다 — 그래서 해석 결과를 엔진이 내려준다.
/// 경로를 구할 수 없으면(HOME 미설정 등) 빈 문자열. 파일의 존재 여부는 보장하지 않는다.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AwsPaths {
    /// 표준 자격증명 파일 (사용자가 다른 파일을 고르지 않았을 때 읽는 곳).
    pub credentials: String,
    /// region 폴백에만 쓰는 config 파일.
    pub config: String,
}

/// 선택된 프로파일의 실제 자격증명 — **비밀 포함**.
///
/// 화면 폼을 채우는 용도로만 쓰고 디스크에 저장하지 않는다 (`StoredAuthConfig` 정책과 동일).
/// 즉 "사용자가 복붙하던 것" 을 파일에서 대신 읽어줄 뿐, 저장 정책은 바뀌지 않는다.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AwsProfileCredentials {
    pub access_key_id: String,
    pub secret_access_key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_token: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
}

/// clusters.json 에 저장하는 형태. 비밀 정보 (password / key_password) 제외.
///
/// `password` 는 매 연결 시 화면에서 직접 입력받는다.
/// 저장된 `StoredAuthConfig` + 화면 입력 password → `KafkaAuth` 로 합성.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct StoredAuthConfig {
    pub protocol: KafkaProtocol,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sasl_mechanism: Option<SaslMechanism>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    /// AWS MSK IAM 전용 region (비밀 아님 — 저장).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub aws_region: Option<String>,
    /// AWS MSK IAM 전용 — 지난번에 고른 자격증명 파일 경로. **비밀이 아니라 "어디서 읽을지" 힌트다**
    /// (키/시크릿/세션토큰은 여전히 저장하지 않는다).
    ///
    /// 왜 저장하나: SSO 임시 자격증명은 수 시간이면 만료되어 `aws sso login` 으로 파일이 갱신된다.
    /// 경로+프로파일명만 기억해 두면 재접속 시 **그 시점의 최신 값을 파일에서 다시 읽어** 채울 수
    /// 있어, SSO OIDC 자동갱신(엔터프라이즈 트랙) 없이도 재선택 마찰이 사라진다.
    /// 빈 값/None = 표준 위치(`~/.aws/credentials`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub aws_credentials_file: Option<String>,
    /// AWS MSK IAM 전용 — 지난번에 고른 프로파일 이름 (예: `807743875261_SRE`). 비밀 아님.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub aws_profile: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ca_cert_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_cert_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_key_path: Option<String>,
}

impl StoredAuthConfig {
    /// 런타임 `KafkaAuth` 에서 비밀 정보를 제거한 저장용 구조체 생성.
    pub fn from_auth(auth: &KafkaAuth) -> Self {
        match auth {
            KafkaAuth::None => StoredAuthConfig {
                protocol: KafkaProtocol::Plaintext,
                ..Default::default()
            },
            KafkaAuth::SaslPlaintext {
                mechanism,
                username,
                ..
            } => StoredAuthConfig {
                protocol: KafkaProtocol::SaslPlaintext,
                sasl_mechanism: Some(*mechanism),
                username: Some(username.clone()),
                ..Default::default()
            },
            KafkaAuth::Ssl {
                ca_cert_path,
                client_cert_path,
                client_key_path,
                ..
            } => StoredAuthConfig {
                protocol: KafkaProtocol::Ssl,
                ca_cert_path: ca_cert_path.clone(),
                client_cert_path: client_cert_path.clone(),
                client_key_path: client_key_path.clone(),
                ..Default::default()
            },
            KafkaAuth::SaslSsl {
                mechanism,
                username,
                aws_region,
                ca_cert_path,
                client_cert_path,
                client_key_path,
                ..
            } => StoredAuthConfig {
                protocol: KafkaProtocol::SaslSsl,
                sasl_mechanism: Some(*mechanism),
                username: Some(username.clone()),
                aws_region: aws_region.clone(),
                ca_cert_path: ca_cert_path.clone(),
                client_cert_path: client_cert_path.clone(),
                client_key_path: client_key_path.clone(),
                // 자격증명 파일/프로파일 힌트는 **런타임 KafkaAuth 가 들고 있지 않다**
                // (연결 시점엔 이미 키가 폼에서 materialize 된 상태라 출처를 모른다).
                // 이 힌트는 화면이 아는 값이라 FE 가 저장용 구조체에 직접 넣어 보낸다.
                ..Default::default()
            },
        }
    }
}
