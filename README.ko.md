
<p align="right">
<b>Open Beta</b> &nbsp;&nbsp;&nbsp;&nbsp;
<a href="README.md"><kbd>English</kbd></a> &nbsp; <kbd><b>한국어</b></kbd> &nbsp; <a href="README.ja.md"><kbd>日本語</kbd></a> &nbsp; <a href="README.zh.md"><kbd>中文</kbd></a>
</p>

<div align="center">

<!-- Korean banner variant is not ready yet; the English one is used here for now, which
     is the more reason to keep the tagline below in text. The wordmark is in the banner,
     so the title is not repeated. Structure follows README.md. -->
<img src="assets/banner-en.png" alt="Kaflow Search — Kafka 메시지를 위한 빠른 데스크톱 검색. 훑지 않고 로컬에 색인합니다. 모든 Kafka · Avro & Protobuf · SASL / TLS / AWS MSK IAM." width="920" />

### 내 데스크톱에 설치하는 Kafka 메시지 검색엔진

**수백만 건 중 조건에 맞는 Kafka 메시지를 몇 초 안에 찾습니다.**

`로컬 인덱싱` · `별도 인프라 불필요` · `가입 없음` · `데이터는 내 컴퓨터 안에`

**[⬇ 다운로드](#다운로드)** · **[웹사이트](https://kaflow-search.whsoul-tools.com/ko/)** · **[설치 안내](#설치)** · [이슈](https://github.com/whsoul/kaflow-search/issues)

</div>

---

## 무엇을 하는 도구인가

|  |  |
|---|---|
| 🔎 **스캔이 아니라 검색** | 수백만 건이어도 키워드·필드 검색이 초 단위로 |
| 🧩 **조건 조합** | Key·Header·중첩 JSON 필드에 `AND` / `OR` / `NOT` |
| 🕒 **시간으로 좁히기** | 정확한 기간 지정, 또는 시계열 차트에서 드릴다운 |
| 👁 **키워드 감시** | 감시어를 걸어두면, 창을 열어둔 채로 그래프가 계속 진행 |
| 🧬 **payload 해석** | JSON · Avro · Protobuf — 로컬 스키마 또는 Confluent Schema Registry |
| 📤 **결과 내보내기** | JSONL / CSV / TSV, 필요 시 gzip·zstd |
| 🔐 **주요 환경에 연결** | **AWS MSK (IAM)** · SASL/SCRAM · SSL/TLS · Confluent Schema Registry |
| 🔌 **오프라인 동작** | 클러스터가 끊겨도 인덱싱된 건 계속 검색 |

![빠릅니다](https://img.shields.io/badge/%EB%B9%A0%EB%A6%85%EB%8B%88%EB%8B%A4-4f46e5?style=flat-square) 수백만 건에서도 초 단위 응답 · 검색할 때마다 Kafka를 다시 읽지 않습니다

![가볍습니다](https://img.shields.io/badge/%EA%B0%80%EB%B3%8D%EC%8A%B5%EB%8B%88%EB%8B%A4-4f46e5?style=flat-square) 데스크톱 앱 하나 · Server·Connector·DB 없음 · 정해둔 한도 안에서 자동 정리

![강력합니다](https://img.shields.io/badge/%EA%B0%95%EB%A0%A5%ED%95%A9%EB%8B%88%EB%8B%A4-4f46e5?style=flat-square) 중첩 JSON·Header 조건 · AND/OR/NOT · 시계열 드릴다운 · Avro/Protobuf · AWS MSK 등 주요 환경 연동

![안전합니다](https://img.shields.io/badge/%EC%95%88%EC%A0%84%ED%95%A9%EB%8B%88%EB%8B%A4-4f46e5?style=flat-square) 클라우드가 아니라 내 컴퓨터에 설치 · 메시지가 컴퓨터를 벗어나지 않음 · 비밀번호 미저장

![클러스터에 부담이 적습니다](https://img.shields.io/badge/%ED%81%B4%EB%9F%AC%EC%8A%A4%ED%84%B0%EC%97%90%20%EB%B6%80%EB%8B%B4%EC%9D%B4%20%EC%A0%81%EC%8A%B5%EB%8B%88%EB%8B%A4-4f46e5?style=flat-square) 읽기 전용이라 쓰지 않고, 인덱싱·싱크 때만 읽습니다. 이후 반복 조회는 클러스터 데이터를 요청하지 않습니다

---

## 데모

<div align="center">
  <img src="assets/kaflow-search-demo.gif" alt="Kaflow Search 데모" width="920" />
</div>

<!-- Video is a GitHub attachment, not a repo file. -->
<div align="center">
  <sub><a href="https://github.com/user-attachments/assets/aee0fc7e-2f7e-484c-bf23-de6455bf9af9">▶ 전체 흐름 영상 보기 (3분)</a> — 모든 단계를 실제 속도로</sub>
</div>

<table>
  <tr>
    <td width="33%"><img src="assets/screenshots/search.png" alt="키워드 검색 결과, 매치된 부분 하이라이트" /><br/><sub><b>검색 결과</b></sub></td>
    <td width="33%"><img src="assets/screenshots/search-builder.png" alt="상세검색 — 중첩 JSON 필드에 AND/OR/NOT 조건, 아래에 매치 결과" /><br/><sub><b>조건 조합</b></sub></td>
    <td width="33%"><img src="assets/screenshots/message-detail.png" alt="메시지 상세 — 디코드된 payload와 헤더" /><br/><sub><b>메시지 상세</b></sub></td>
  </tr>
</table>

---

## 다운로드

| 운영체제 | 아키텍처 | 파일 |
|---|---|---|
| macOS | Apple Silicon | `.dmg` |
| macOS | Intel | `.dmg` |
| Windows | x64 | [`.exe` / `.msi` — **Preview, 검증 중**](https://github.com/whsoul/kaflow-search/releases/tag/v0.1.2_win) |
| Linux | — | 수요가 모이면 제공 — [의견 더하기](https://github.com/whsoul/kaflow-search/issues/6) |

**[⬇ 최신 릴리스 받기](https://github.com/whsoul/kaflow-search/releases/latest)** — 설치 파일은 여기서만 받으세요.

**요구 사항** — macOS 11 (Big Sur) 이상 또는 Windows 10 이상 (x64) · Apache Kafka 2.4 이상 ([어떤 환경인지](#지원-환경))

| | |
|---|---|
| **설치 공간** | 약 100MB |
| **여유 공간** | 설치 공간과 **별도로**, 데이터 인덱싱을 위해 **클러스터당 10GB 이상**이 있어야 안정적입니다 (클러스터 2개 = 20GB 이상). 인덱스 한도는 설정에서 조정할 수 있습니다 |
| **디스크 종류** | **SSD 권장.** 인덱싱은 쓰기가 많아 HDD에서도 동작하지만 눈에 띄게 느립니다 |

별도의 데이터베이스나 서비스는 필요 없습니다. Windows에서 실행 직후 종료된다면
[Windows 설치·실행 문제 해결](https://github.com/whsoul/kaflow-search/issues/5#issuecomment-5406572758)을 확인해 주세요.

> 🔔 **새 버전이 나오면 알림을 받으시려면** — 이 저장소 상단의 **[Watch ▾] → Custom → ☑ Releases**
> 를 켜두시면 릴리스가 올라올 때마다 알려드립니다. (준비 중인 플랫폼을 기다리시는 분께도 유용합니다.)

## 설치

> ### ⚠️ 설치할 때 경고가 뜹니다. 정상입니다.
>
> Kaflow Search는 아직 **Apple·Windows의 개발자 인증을 받지 않았습니다.** 유료 인증서가 필요한데
> 아직 구매하지 못했습니다. 그래서 OS가 "확인되지 않은 개발자"로 표시하는 것이지,
> **앱에서 유해한 것이 발견됐다는 뜻이 아닙니다.** 아래 절차대로 **한 번만 허용**하시면
> 그다음부터는 평소처럼 실행됩니다.
>
> *(원하시면 받으신 파일이 변조되지 않았는지 [직접 확인](#데이터와-안전성)하실 수도 있습니다. 필수는 아닙니다.)*


<details open>
<summary><b>macOS</b> — 시스템 설정에서 "확인 없이 열기"</summary>

1. `.dmg`를 열고 **Kaflow Search**를 응용 프로그램 폴더로 끌어다 놓습니다.

   > 💡 **DMG 안에서 바로 실행하지 마세요.** 옮긴 뒤 응용 프로그램에서 실행해야 허용 절차를
   > 두 번 반복하지 않습니다. (허용은 파일 하나에만 적용돼서, DMG 안의 앱을 허용해도 복사본에는
   > 이어지지 않습니다.)

2. 응용 프로그램에서 앱을 실행합니다. 아래 경고가 뜨면 **확인**을 누릅니다.

   > `'Kaflow Search'은(는) Apple에서 악성 소프트웨어가 있는지 확인할 수 없기 때문에 열 수 없습니다.`

3. **시스템 설정 → 개인 정보 보호 및 보안**으로 가서, **보안** 항목 아래의
   **[확인 없이 열기]** 를 누릅니다. 그다음 뜨는 창에서 한 번 더 확인하면 실행됩니다.

   > `확인된 개발자가 등록한 응용 프로그램이 아니기 때문에 'Kaflow Search' 사용을 차단했습니다.`

   *(이 버튼은 2번에서 차단된 **직후에만** 나타납니다. 안 보이면 앱을 한 번 더 실행해 보세요.)*

**환경에 따라 이 확인을 두 번 요구할 수 있습니다** (설치 직후 한 번, 첫 실행에서 한 번).
잘못된 것이 아니니 같은 방법으로 한 번 더 허용해 주세요. 이후로는 묻지 않습니다.

<br/>

**한 번에 끝내려면** — 위 과정을 건너뛰고 터미널에서 아래 한 줄을 실행한 뒤 평소처럼 열면 됩니다.

```bash
xattr -dr com.apple.quarantine "/Applications/Kaflow Search.app"
```

*(설치한 버전마다 한 번씩만 하면 됩니다. 경고 문구는 macOS 버전에 따라 조금씩 다릅니다.)*

</details>

<details open>
<summary><b>Windows</b> — "알 수 없는 게시자" 경고에서 계속 진행</summary>

Windows용 빌드는 현재 **Preview로 제공되며 검증 중**입니다.

설치할 때 Windows가 **알 수 없는 게시자** 또는 Microsoft Defender SmartScreen 경고를 표시할 수 있습니다.
이는 macOS와 마찬가지로 Kaflow Search가 아직 유료 코드 서명 인증서를 사용하지 않기 때문이며,
앱에서 유해한 것이 발견됐다는 뜻은 아닙니다. **추가 정보 → 실행**을 선택해 설치를 계속해 주세요.

설치 또는 실행에 문제가 있거나 앱이 실행 직후 종료된다면
[Windows 설치·실행 문제 해결](https://github.com/whsoul/kaflow-search/issues/5#issuecomment-5406572758)을 확인해 주세요.

</details>

## 시작하기

| | 단계 | 참고 |
|---|---|---|
| **1** | 클러스터 연결 | 비밀번호는 **저장하지 않습니다** — 연결할 때마다 입력 |
| **2** | 인덱싱할 토픽 선택 | 실제 메시지 샘플로 역직렬화 방식·정리 정책을 자동 추천 |
| **3** | 인덱싱 | 토픽을 한 번 읽습니다. 진행 중에도 앱은 계속 사용 가능 |
| **4** | 검색 | 키워드, 또는 Key·Header·JSON 필드 조건 |

<details>
<summary><b>AWS MSK (IAM 인증)에 연결하려면</b></summary>

MSK는 별도 메뉴가 아니라 **SASL 방식 목록 안에** 있습니다. 연결 화면에서:

1. **Security protocol** → `SASL_SSL` 선택 *(MSK는 TLS 필수라 `SASL_PLAINTEXT`로는 연결되지 않습니다)*
2. **SASL mechanism** → `AWS MSK IAM (IAM access key)` 선택
3. `~/.aws/credentials`가 있으면 **프로파일 선택 콤보**가 나타납니다. 고르면 Access Key·Region이
   자동으로 채워집니다. *(프로파일이 하나면 콤보 없이 바로 적용됩니다.)*
4. 없거나 SSO 전용 프로파일이면 직접 입력 — **AWS Access Key ID** / **AWS Secret Access Key** /
   **Region**(예: `ap-northeast-2`, MSK 클러스터가 있는 리전)

키는 **저장되지 않고** 그 연결에만 쓰입니다. 연결마다 다시 입력하거나 프로파일에서 다시 불러옵니다.

> `Access denied`로 실패하면 연결 실패 창에서 **최소 권한 IAM 정책 JSON**을 복사할 수 있습니다.
> MSK는 키 오타·리전 불일치·권한 부족·권한 경계가 **모두 같은 메시지**로 나오므로, 앱이 원인을
> 단정하지 않고 확인할 항목을 함께 안내합니다.

</details>

---

# 상세 정보

## 왜 로컬 인덱스인가

Kafka 클러스터가 크다고 해서 전체 데이터를 내 컴퓨터에 복제하는 방식이 아닙니다. 필요한 토픽만
직접 선택하고, 설정된 저장 공간 예산 안에서 로컬 검색 인덱스를 관리합니다.

| | |
|---|---|
| **문제** | Kafka는 로그이지 DB가 아니라, "이 주문번호가 든 메시지"를 찾아낼 수단이 없음 |
| **Kaflow가 하는 일** | 토픽을 한 번 읽어 로컬 인덱스 생성. 이후 검색은 Kafka가 아니라 인덱스에서 |
| **선택적 인덱싱** | 필요한 토픽만 직접 선택. 클러스터의 모든 토픽을 자동으로 인덱싱하지 않음 |
| **제한된 로컬 저장 공간** | 설정 가능한 저장 공간 예산과 정리 정책으로 로컬 인덱스가 무한히 커지지 않게 관리 |
| **범위** | **토픽에 아직 남아 있는** 것만 — Kafka 보존기간을 그대로 따라감. 보관 도구가 아님 |
| **관리 도구와의 관계** | 겹치지도, 충돌하지도 않음. 운영은 쓰던 도구로, Kaflow는 **찾을 때** |

## 데이터와 안전성

| | |
|---|---|
| **메시지·인덱스·검색어** | 내 컴퓨터에서만 처리. Kaflow 서버로 전송되지 않음 |
| **비밀번호·토큰·시크릿 키** | **저장하지 않음.** 세션 동안 메모리에만, 연결할 때마다 재입력 |
| **로컬 저장 항목** (`~/.kaflow/`) | 비밀이 아닌 정보만 — 프로토콜, 인증 방식, 사용자명, 인증서 경로 |
| **클러스터 쓰기** | **없음.** 읽기 전용: `Metadata`, `ApiVersions`, `DescribeConfigs`, `DescribeTopicPartitions`, `Fetch`, `ListOffsets`. `Produce`도 관리 조작도 없어 토픽·컨슈머 그룹·ACL을 바꿀 **수 없음** |
| **클러스터 읽기 부담** | 인덱싱과 증분 싱크 때만 읽습니다. 검색·탐색·내보내기는 **로컬 인덱스에서 처리**되어 반복해도 클러스터에 요청이 가지 않습니다 (메시지 원문을 여는 상세 보기는 예외) |
| **Kaflow로 전송** | 시작 시, 그리고 앱을 켜둔 동안 몇 시간마다 버전 지원·공지 확인용으로: 임의 인스턴스 ID · 해시된 기기 ID · 앱 버전 · OS/아키텍처 · UI 언어. Kafka 설정이나 메시지 정보는 없음 |
| **진단 리포트** | 로컬 생성. 직접 보내기를 선택할 때만 전송 |

보관 기간을 포함한 상세: **[개인정보 처리방침](legal/PRIVACY.ko.md)**

<details>
<summary><b>받은 파일이 변조되지 않았는지 확인하기</b> (선택)</summary>

> **설치나 실행에 필요한 절차가 아닙니다.** 하지 않아도 앱은 정상 동작합니다.
> 보안을 더 챙기고 싶은 분을 위한 부가 확인 수단입니다.

릴리스마다 **`SHA256SUMS`** 파일과, 릴리스 노트 본문의 **SHA-256** 표가 함께 제공됩니다.
받으신 설치 파일의 지문을 계산해 이 값과 비교하면, 파일이 **다운로드 중 손상됐거나 중간에서
바꿔치기된 것이 아닌지** 확인할 수 있습니다.

```bash
# macOS — SHA256SUMS 를 같은 폴더에 두고 한 번에 대조
shasum -a 256 -c SHA256SUMS
```

```powershell
# Windows — 값을 계산해 릴리스 노트의 값과 눈으로 비교
Get-FileHash <파일> -Algorithm SHA256
```

값이 다르면 그 파일은 쓰지 말고 삭제한 뒤 다시 받으세요.

⚠️ **한계도 알아두세요.** 파일과 체크섬이 같은 페이지에 있으므로, 릴리스 페이지 자체가 장악된
경우에는 둘 다 바뀌어 이 방법으로 걸러낼 수 없습니다. 그런 경우까지 막아주는 것은 개발자 인증
(코드 서명)이며, 현재는 없는 상태입니다.

</details>

<details>
<summary><b>연결 전 확인할 것</b></summary>

- 연결하려는 클러스터·토픽에 읽기 권한이 있는지 확인
- 조직의 보안·데이터 취급 정책 준수
- 로컬 인덱스와 내보낸 파일은 디스크에 **암호화되지 않은 채** 저장됨 — 보관에 유의
- 가능하면 최소 권한 Kafka 계정 사용
- 로그 공유 전 인증 정보와 메시지 내용 제거

</details>

## 지원 환경

| | |
|---|---|
| **보안 프로토콜** | `PLAINTEXT` · `SSL` · `SASL_PLAINTEXT` · `SASL_SSL` |
| **SASL 방식** | `PLAIN` · `SCRAM-SHA-256` · `SCRAM-SHA-512` · `OAUTHBEARER` · **AWS MSK IAM** |
| **인증서** | PEM · PKCS#12 (`.p12` / `.pfx`) |
| **Schema Registry** | Confluent (Avro·Protobuf, 스키마 참조 포함) |
| **Kafka 버전** | 목표 **2.4 ~ 4.x** · **검증 완료:** Kafka 2.4.x(로컬), AWS MSK(IAM) |

그 외 버전과 매니지드 서비스도 동작할 것으로 보지만 아직 검증하지 못했습니다 —
[써보셨다면 알려주세요](https://github.com/whsoul/kaflow-search/issues/new?template=connection_report.yml). 연결 성공 여부는 버전만이
아니라 브로커 설정·네트워크 정책·제공업체별 인증 구현에도 영향을 받습니다.

<details>
<summary><b>검색할 수 있는 것</b></summary>

| 대상 | 예 |
|---|---|
| 메시지 Key | `K.orderId` — 값 자체 또는 중첩 JSON |
| Payload | `P.customer.email` — 일반 텍스트 또는 중첩 JSON |
| Header | `H.trace-id` |
| 배열 | `P.items[*].sku` — 하나의 경로로 통합 |

| 조건 | |
|---|---|
| 매칭 | 정확히 일치 · 접두(prefix) · 필드 안 사전순 범위 |
| 논리 | `AND` / `OR` / `NOT`, `OR`의 최소 일치 개수 지정 |
| 필터 | 타임스탬프 범위 · 파티션/오프셋 범위 |
| 어절 | 어절 지정한 필드는 단어 단위 매칭 |

| 탐색 | |
|---|---|
| 차트 | 시계열 → 해당 구간 메시지로 드릴다운 |
| 트리 | 오프셋 트리 · 날짜 트리 · 파티션 배치 · 최근 메시지 |
| 상세 | JSON 하이라이트, 복사 |
| 내보내기 | JSONL / CSV / TSV × gzip / zstd — 로컬 인덱스 기반이라 오프라인 동작 |

</details>

<details>
<summary><b>리소스 한도</b></summary>

디스크, 규모, 인덱스 보관·정리 방식에 각각 한도가 있습니다. 내 컴퓨터의 안정성을 해치지 않는
선에서 정한 값입니다. 지금 적용되는 값은 앱의 **설정** 화면에 나오며, 앱은 거기 표시된 값을
그대로 지킵니다.

인덱싱 시간과 디스크 사용량은 메시지 수·크기·파티션 수·인덱싱 필드 수·디스크 성능에 따라
달라지며, 일정한 성능을 보장하지 않습니다.

</details>

## 프로젝트 현황

**0.1.0 — 오픈 베타.** 실제 업무에 쓸 수 있습니다.

**알려진 제한**

| | |
|---|---|
| 한 번에 한 토픽 | 여러 토픽 통합 검색이 다음 기능 |
| 문자열 기준 범위 | 숫자·날짜 범위 검색(`금액 50~100`)은 아직 없음 |
| 수동 업데이트 | 자동 업데이트 없음 — 새 버전은 직접 받아 설치 |
| 미서명 | [설치](#설치) 참조 |

> ⚠️ 결과가 없다는 건 **내 로컬 인덱스 안에서** 일치하는 게 없다는 뜻입니다. 그 메시지가 Kafka에
> 없었다는 증명은 아닙니다 — 인덱싱 이전 구간이거나, 인덱싱한 필드·범위 밖일 수 있습니다.

## 피드백

[버그 제보](https://github.com/whsoul/kaflow-search/issues/new?template=bug_report.yml) ·
[연결 문제](https://github.com/whsoul/kaflow-search/issues/new?template=connection_report.yml) ·
[전체 이슈](https://github.com/whsoul/kaflow-search/issues)

각 양식은 판단에 실제로 필요한 것만 묻습니다. 연결 문제라면 사용하는 서비스, Kafka 버전,
인증 방식, 그리고 같은 컴퓨터에서 다른 Kafka 클라이언트는 연결되는지입니다.

> 🔒 **비밀번호, 토큰, 개인키, 인증서, 실제 업무 메시지를 공개 이슈에 올리지 마세요.**

## 프로젝트 후원

독립적으로 개발·운영됩니다. 후원은 자발적이며 구매·구독이나 특정 기능·일정에 대한 약속이
아닙니다. 후원 여부와 관계없이 무료 이용 범위는 동일합니다.

**향후 후원이 쓰이는 곳:** macOS·Windows 코드 서명 인증서 — [설치 경고](#설치)가 아직 남아
있는 이유입니다. 그 밖에 더 많은 플랫폼·Kafka 버전 테스트, 배포 인프라에 씁니다.

## 이 저장소

Kaflow Search의 공식 공개 저장소 — 릴리스, 문서, 이슈, 로드맵.

| | |
|---|---|
| **source-available, 오픈소스 아님** | 실제 프론트엔드와 검색 엔진 소스는 **여기 없습니다** |
| **여기 있는 것** | 공개 API 계약 · mock 엔진 · 데스크톱 셸 · 실행 가능한 데모 빌드 |
| **이유** | 투명성과 평가용. 특정 파일에 명시가 없는 한 오픈소스 라이선스 미적용 — [LICENSE](LICENSE) 참조 |
| **기여** | 코드 기여는 받지 않습니다. 이슈와 피드백은 환영합니다 |

**법적 문서** — [LICENSE](LICENSE) (저장소 이용 조건) ·
[EULA](legal/EULA.ko.md) ([English](legal/EULA.md)) ·
[개인정보 처리방침](legal/PRIVACY.ko.md) ([English](legal/PRIVACY.md)) ·
[제3자 라이선스 고지](legal/THIRD_PARTY_NOTICES.md)

proprietary 소프트웨어이며, EULA에 따라 현재 **개인용·조직 내부 업무용으로 무료** 제공됩니다.
재판매, 수정, 역공학, 무단 재배포, 다른 제품에 포함하는 행위는 제한됩니다. 향후 별도 기능·서비스에는
다른 조건이 적용될 수 있으며 적용 전에 안내합니다.

앱 화면은 **English · 한국어 · 日本語 · 中文** 을 지원합니다.

**상표** — Apache Kafka와 Kafka는 Apache Software Foundation의 상표입니다. Kaflow Search는 ASF와
관련이 없으며 승인·후원받거나 제휴한 제품이 아닙니다. 그 밖의 제품명과 상표는 각 소유자의 자산입니다.

---

<div align="center">

**Kaflow Search** — 내 데스크톱에 설치하는 Kafka 메시지 검색엔진

[다운로드](https://github.com/whsoul/kaflow-search/releases/latest) ·
[웹사이트](https://kaflow-search.whsoul-tools.com/ko/) ·
[이슈](https://github.com/whsoul/kaflow-search/issues) ·
[개인정보](legal/PRIVACY.ko.md)

Copyright © 2026 Whsoul Tools. Kaflow Search is a product of Whsoul Tools.

</div>
