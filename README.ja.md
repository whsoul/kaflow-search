
<p align="right">
<a href="README.md"><kbd>English</kbd></a> &nbsp; <a href="README.ko.md"><kbd>한국어</kbd></a> &nbsp; <kbd><b>日本語</b></kbd> &nbsp; <a href="README.zh.md"><kbd>中文</kbd></a>
</p>

<div align="center">

<!-- Japanese banner variant is not ready yet; the English one is used here for now, which
     is the more reason to keep the tagline below in text. The wordmark is in the banner,
     so the title is not repeated. Structure follows README.md. -->
<img src="assets/banner-en.png" alt="Kaflow Search — Kafka メッセージのための高速なデスクトップ検索。スキャンではなく、ローカルにインデックスします。あらゆる Kafka · Avro & Protobuf · SASL / TLS / AWS MSK IAM。" width="920" />

### Kafka メッセージのためのデスクトップ検索エンジン

**数百万件のなかから目的のメッセージを、手元のマシンで数秒のうちに。**

`ローカルインデックス` · `追加インフラ不要` · `サインアップ不要` · `データは手元から出ない`

**[⬇ ダウンロード](#ダウンロード)** · **[インストール手順](#インストール)** · [Issue](https://github.com/whsoul/kaflow-search/issues)

</div>

---

## できること

|  |  |
|---|---|
| 🔎 **スキャンではなく検索** | 数百万行のインデックスに対して、キーワード検索もフィールド検索も数秒で |
| 🧩 **条件の組み合わせ** | キー・ヘッダー・ネストした JSON フィールドに `AND` / `OR` / `NOT` |
| 🕒 **時間で絞り込む** | 期間を指定、または時系列チャートからドリルダウン |
| 🧬 **ペイロードのデコード** | JSON · Avro · Protobuf — ローカルスキーマまたは Confluent Schema Registry |
| 📤 **結果のエクスポート** | JSONL / CSV / TSV、必要なら gzip・zstd |
| 🔐 **今の環境につながる** | **AWS MSK (IAM)** · SASL/SCRAM · SSL/TLS · Confluent Schema Registry |
| 🔌 **オフラインでも動く** | クラスタに届かなくても、インデックス済みのデータは検索できます |

![速い](https://img.shields.io/badge/%E9%80%9F%E3%81%84-4f46e5?style=flat-square) 数百万行を数秒で · 検索のたびに Kafka を読み直しません

![軽い](https://img.shields.io/badge/%E8%BB%BD%E3%81%84-4f46e5?style=flat-square) デスクトップアプリひとつ · サーバーもコネクタもデータベースも不要 · 決めた上限のなかで自動整理

![強力](https://img.shields.io/badge/%E5%BC%B7%E5%8A%9B-4f46e5?style=flat-square) ネストした JSON とヘッダー条件 · AND/OR/NOT · 時系列ドリルダウン · Avro/Protobuf · AWS MSK などのマネージドクラスタに接続

![安全](https://img.shields.io/badge/%E5%AE%89%E5%85%A8-4f46e5?style=flat-square) クラウドサービスではなく手元のマシンにインストール · メッセージが外に出ません · パスワードは保存しません

![クラスタにやさしい](https://img.shields.io/badge/%E3%82%AF%E3%83%A9%E3%82%B9%E3%82%BF%E3%81%AB%E3%82%84%E3%81%95%E3%81%97%E3%81%84-4f46e5?style=flat-square) 読み取り専用で、読むのはインデックス作成と同期のときだけ。以降は何度検索してもクラスタには何も要求しません

---

## デモ

<div align="center">
  <img src="assets/kaflow-search-demo.gif" alt="Kaflow Search デモ" width="920" />
</div>

<!-- Video is a GitHub attachment, not a repo file. -->
<div align="center">
  <sub><a href="https://github.com/user-attachments/assets/aee0fc7e-2f7e-484c-bf23-de6455bf9af9">▶ 全体のウォークスルーを見る (3 分)</a> — すべての手順を実際の速度で</sub>
</div>

<!-- Screenshots still to come. Add these, then uncomment:
       assets/screenshots/{search,search-builder,message-detail,timeseries}.png
     Left commented so nothing renders broken.

<table>
  <tr>
    <td width="50%"><img src="assets/screenshots/search.png" alt="検索結果" /><br/><sub><b>検索結果</b></sub></td>
    <td width="50%"><img src="assets/screenshots/search-builder.png" alt="条件ビルダー" /><br/><sub><b>条件ビルダー</b></sub></td>
  </tr>
  <tr>
    <td width="50%"><img src="assets/screenshots/message-detail.png" alt="メッセージ詳細" /><br/><sub><b>メッセージ詳細</b></sub></td>
    <td width="50%"><img src="assets/screenshots/timeseries.png" alt="時系列" /><br/><sub><b>時系列ドリルダウン</b></sub></td>
  </tr>
</table>
-->

---

## ダウンロード

| OS | アーキテクチャ | ファイル |
|---|---|---|
| macOS | Apple Silicon | `.dmg` |
| macOS | Intel | `.dmg` |
| Windows | x64 | ⏳ 作業中 — [公開されたら通知を受け取る](https://github.com/whsoul/kaflow-search/issues/5) |
| Linux | — | ご要望が集まれば提供 — [賛同する](https://github.com/whsoul/kaflow-search/issues/6) |

**[⬇ 最新リリースを入手](https://github.com/whsoul/kaflow-search/releases/latest)** — ダウンロードはここからのみお願いします。

**動作要件** — macOS 11 (Big Sur) 以降 · Apache Kafka 2.4 以降 ([対応環境](#動作環境))

| | |
|---|---|
| **インストールサイズ** | 約 100 MB |
| **空き容量** | **それとは別に**、ローカルインデックス用としてクラスタあたり **10 GB 以上**（2 クラスタなら 20 GB 以上）。インデックスの上限は設定で変更できます |
| **ディスク** | **SSD 推奨。** インデックス作成は書き込みが多く、HDD でも動きますが目に見えて遅くなります |

ほかに入れるものはありません — ランタイムもデータベースもサービスも不要です。

> 🔔 **新しいバージョンを知りたいときは** このリポジトリ上部の **[Watch ▾] → Custom → ☑ Releases**
> を使うと、リリースのたびに通知が届きます。まだ作業中のプラットフォームを待っている場合にも便利です。

## インストール

> ### ⚠️ OS が警告を出します。想定どおりです。
>
> Kaflow Search は Apple や Microsoft の**開発者認証をまだ受けていません**。有料の証明書が必要で、
> このプロジェクトはまだ購入していないためです。そのため OS は「開発元が確認できないアプリ」として
> 警告します。**有害なものが見つかったという意味ではありません。** 下の手順で**一度だけ許可**すれば、
> 以降は普通に起動します。
>
> *(必要なら[ダウンロードしたファイルが改ざんされていないか確認](#データと安全性)することもできます。必須ではありません。)*


<details open>
<summary><b>macOS</b> — システム設定 →「このまま開く」</summary>

1. `.dmg` を開き、**Kaflow Search** をアプリケーションフォルダにドラッグします。

   > 💡 **DMG のなかから起動しないでください。** 先に移動して、アプリケーションフォルダから
   > 起動します。そうしないと許可の操作を 2 回行うことになります。（許可はファイル 1 つに対して
   > 働くため、DMG のなかのコピーを許可しても、取り出した方には引き継がれません。）

2. アプリケーションフォルダから起動します。次の表示が出たら **OK** を押します。

   > `“Kaflow Search”は、Appleによる悪質なソフトウェアかどうかのチェックができないため開けません。`

3. **システム設定 → プライバシーとセキュリティ**を開き、**セキュリティ**の項目で
   **［このまま開く］**をクリックします。続くダイアログでもう一度確認します。

   > `“Kaflow Search”は開発元を確認できないため、使用がブロックされました。`

   *(このボタンは手順 2 でブロックされた**直後**にだけ現れます。見当たらない場合は、もう一度アプリを起動してください。)*

**環境によっては 2 回聞かれます** — インストール直後に一度、初回起動時に一度です。異常ではないので、
同じように許可してください。それ以降は聞かれません。

<br/>

**すべて省略したい場合**は、代わりにターミナルで次を 1 回実行し、あとは普通にアプリを開いてください。

```bash
xattr -dr com.apple.quarantine "/Applications/Kaflow Search.app"
```

*(インストールしたバージョンごとに 1 回。文言は macOS のバージョンによって多少異なります。)*

</details>

## はじめかた

| | 手順 | 補足 |
|---|---|---|
| **1** | クラスタに接続 | パスワードは**保存しません** — 接続のたびに入力します |
| **2** | インデックスするトピックを選ぶ | デシリアライザと整理ポリシーは実際のサンプルから自動で提案されます |
| **3** | インデックス作成 | トピックを 1 回読みます。実行中もアプリは使えます |
| **4** | 検索 | キーワード、またはキー / ヘッダー / JSON フィールドへの条件で |

<details>
<summary><b>AWS MSK (IAM 認証) に接続する</b></summary>

MSK は独立したメニューではなく、**SASL メカニズムの一覧のなか**にあります。接続画面で:

1. **Security protocol** → `SASL_SSL` *(MSK は TLS が必須です。`SASL_PLAINTEXT` では接続できません)*
2. **SASL mechanism** → `AWS MSK IAM (IAM access key)`
3. `~/.aws/credentials` があれば**プロファイル選択**が表示されます。選ぶとアクセスキーとリージョンが
   入力されます。*(プロファイルが 1 つなら選択なしでそのまま適用されます。)*
4. それ以外の場合や SSO のみのプロファイルでは、ご自身で入力します — **AWS Access Key ID** /
   **AWS Secret Access Key** / **リージョン**（例: `ap-northeast-2`、MSK クラスタがある場所）

キーは**保存されません**。その接続にだけ使われ、次回はもう一度入力するかプロファイルから読み込みます。

> `Access denied` で失敗する場合、失敗ダイアログに**最小権限の IAM ポリシー JSON** が表示され、
> コピーできます。MSK はキーの打ち間違い・リージョン違い・権限不足・アクセス許可境界のいずれでも
> 同じメッセージを返すため、アプリは原因を決めつけず、確認すべき項目を並べます。

</details>

---

# 詳細

## なぜローカルインデックスなのか

| | |
|---|---|
| **課題** | Kafka はログであってデータベースではなく、「この注文 ID のメッセージ」を引く手段がありません |
| **Kaflow がすること** | 各トピックを一度読んでローカルにインデックスを作ります。以降の検索は Kafka ではなくインデックスに当たります |
| **範囲** | 検索できるのは**トピックにまだ残っているもの**です。Kafka の保持期間をそのまま映すもので、アーカイブではありません |
| **管理ツールとの関係** | 重なりません、ぶつかりません。クラスタの運用は今お使いのもので、**探す**のは Kaflow で |

## データと安全性

| | |
|---|---|
| **メッセージ・インデックス・検索語** | 手元のマシンで処理します。Kaflow のサーバーに送信されることはありません |
| **パスワード・トークン・シークレットキー** | **保存しません。** セッションのあいだメモリ上に保持し、接続のたびに入力します |
| **ローカルに保存されるもの** (`~/.kaflow/`) | 秘密でないメタデータのみ — プロトコル、メカニズム、ユーザー名、証明書のパス |
| **クラスタへの書き込み** | **ありません。** 読み取り専用です: `Metadata`, `ApiVersions`, `DescribeConfigs`, `DescribeTopicPartitions`, `Fetch`, `ListOffsets`。`Produce` も管理系の呼び出しもないため、トピック・コンシューマグループ・ACL を変更**できません** |
| **クラスタへの読み取り負荷** | インデックス作成と同期のときだけです。検索・探索・エクスポートはすべてローカルインデックス上で動くので、繰り返してもクラスタには何も送りません（メッセージの生ペイロードを開くときだけは例外です） |
| **Kaflow に送られるもの** | 起動時のみ、バージョンサポートとお知らせのために: ランダムなインスタンス ID · ハッシュ化したマシン ID · アプリのバージョン · OS / アーキテクチャ · UI の言語。Kafka の構成やメッセージについては何も送りません |
| **診断レポート** | ローカルで生成され、送信するかどうかはご自身で選べます |

保持期間を含む詳細: **[PRIVACY.md](legal/PRIVACY.md)**

<details>
<summary><b>ダウンロードしたファイルが改ざんされていないか確認する</b>（任意）</summary>

> **インストールや実行に必須ではありません。** 飛ばしても問題なく動きます。
> 念のため確かめたい方のための項目です。

すべてのリリースには **`SHA256SUMS`** ファイルが付属し、リリースノートにも同じ **SHA-256** 値が
載っています。ダウンロードしたファイルの指紋を計算して突き合わせれば、**転送中に壊れていないか、
途中ですり替えられていないか**がわかります。

```bash
# macOS — SHA256SUMS を同じフォルダに置いて、まとめて照合
shasum -a 256 -c SHA256SUMS
```

```powershell
# Windows — 値を計算し、リリースノートの値と目視で比較
Get-FileHash <file> -Algorithm SHA256
```

値が一致しない場合、そのファイルは使わずに削除して、もう一度ダウンロードしてください。

⚠️ **限界も知っておいてください。** ファイルとチェックサムは同じページにあるため、リリースページ
そのものが侵害された場合は両方が同時に書き換わり、この確認では気づけません。そこを守るのが
開発者認証（コード署名）であり、このプロジェクトはまだ持っていません。

</details>

<details>
<summary><b>接続する前に</b></summary>

- そのクラスタとトピックを読む権限があることを確認してください
- 所属組織のセキュリティおよびデータ取り扱いポリシーに従ってください
- ローカルのインデックスとエクスポートはディスク上で**暗号化されていません** — 保管場所にご注意ください
- 権限を絞った Kafka アカウントをお使いください
- ログを共有するときは、認証情報とメッセージ本文を取り除いてください

</details>

## 動作環境

| | |
|---|---|
| **セキュリティプロトコル** | `PLAINTEXT` · `SSL` · `SASL_PLAINTEXT` · `SASL_SSL` |
| **SASL メカニズム** | `PLAIN` · `SCRAM-SHA-256` · `SCRAM-SHA-512` · `OAUTHBEARER` · **AWS MSK IAM** |
| **証明書** | PEM · PKCS#12 (`.p12` / `.pfx`) |
| **Schema Registry** | Confluent (Avro、Protobuf、スキーマ参照を含む) |
| **Kafka バージョン** | 対象は **2.4 – 4.x** · **確認済み:** Kafka 2.4.x（ローカル）、AWS MSK (IAM) |

それ以外のバージョンやマネージドサービスも動作する見込みですが、まだ確認できていません —
[結果を教えてください](https://github.com/whsoul/kaflow-search/issues/new?template=connection_report.yml)。接続できるかどうかは
バージョンだけでなく、ブローカー設定・ネットワークポリシー・プロバイダ固有の認証にも左右されます。

<details>
<summary><b>検索できる対象</b></summary>

| 対象 | 例 |
|---|---|
| メッセージキー | `K.orderId` — 素の値でもネストした JSON でも |
| ペイロード | `P.customer.email` — プレーンテキストでもネストした JSON でも |
| ヘッダー | `H.trace-id` |
| 配列 | `P.items[*].sku` — 1 つのパスに平坦化 |

| 条件 | |
|---|---|
| マッチ | 完全一致 · 前方一致 · フィールド内の辞書順範囲 |
| 論理 | `AND` / `OR` / `NOT`、`OR` には最小一致数の指定も |
| フィルタ | タイムスタンプ範囲 · パーティション / オフセット範囲 |
| 単語 | トークン化を指定したフィールドは単語単位でマッチ |

| 探索 | |
|---|---|
| チャート | 時系列から該当メッセージへドリルダウン |
| ツリー | オフセットツリー · 日付ツリー · パーティション配置 · 最新メッセージ |
| 詳細 | JSON ハイライト、コピー |
| エクスポート | JSONL / CSV / TSV × gzip / zstd — ローカルインデックス上で動くのでオフラインでも |

</details>

<details>
<summary><b>リソース上限</b></summary>

ディスク、規模、インデックスの保持と整理のしかたに、それぞれ上限があります。お使いのマシンに
負担をかけない範囲で決めた値です。現在適用されている値はアプリの**設定**画面に表示され、
アプリは表示どおりに動きます。

インデックス作成の時間とディスク使用量は、件数・サイズ・パーティション数・インデックス対象
フィールド・ディスク速度によって変わるため、一定の性能を保証するものではありません。

</details>

## プロジェクトの状況

**バージョン 0.1.0 — 最初の公開リリースです。** 日常の作業に使えますが、まだ初期であり、バージョン番号もそう言っています。

**既知の制限**

| | |
|---|---|
| 一度に 1 トピック | 複数トピックを横断する検索が次の機能です |
| 範囲はテキスト基準 | 数値・日付の範囲検索（`amount 50–100`）はまだありません |
| 手動アップデート | 自動更新はなく、新しいバージョンは手で入れ替えます |
| 未署名 | [インストール](#インストール)を参照 |

> ⚠️ 結果が 0 件でも、それは**ローカルインデックスのなかで**一致しなかったという意味であり、
> Kafka にそのメッセージが存在しなかったという意味ではありません。インデックスより前のものか、
> インデックス対象にしたフィールドや範囲の外にあるのかもしれません。

## フィードバック

[バグを報告](https://github.com/whsoul/kaflow-search/issues/new?template=bug_report.yml) ·
[接続の問題](https://github.com/whsoul/kaflow-search/issues/new?template=connection_report.yml) ·
[機能を要望](https://github.com/whsoul/kaflow-search/issues/new?template=feature_request.yml) ·
[すべての Issue](https://github.com/whsoul/kaflow-search/issues)

各フォームは、判断に本当に必要なことだけを尋ねます。接続の問題であれば、お使いのサービス、
Kafka のバージョン、認証方式、そして同じマシンからほかの Kafka クライアントが接続できるか、です。

> 🔒 **パスワード・トークン・秘密鍵・証明書・実業務のメッセージを公開 Issue に載せないでください。**

## プロジェクトを支援する

個人で開発・維持しています。ご支援は任意で、購入でも定期契約でもなく、特定の機能や時期を
約束するものでもありません。無償で使える条件はどちらでも変わりません。

**ご支援の使い道:** macOS と Windows のコード署名証明書 — [インストール時の警告](#インストール)が
まだ出るのはこのためです。そのほかに、より多くのプラットフォームと Kafka バージョンでの検証、
リリース基盤に充てます。

## このリポジトリ

公式の公開リポジトリです — リリース、ドキュメント、Issue、ロードマップ。

| | |
|---|---|
| **ソース公開型で、オープンソースではありません** | 製品版のフロントエンドと検索エンジンはここに**ありません** |
| **ここにあるもの** | 公開 API の契約 · モックエンジン · デスクトップシェル · 動かせるデモビルド |
| **理由** | 透明性と評価のためです。ファイルに明記がない限り、オープンソースライセンスは適用されません — [LICENSE](LICENSE) をご覧ください |
| **コントリビューション** | コードの受け入れはしていません。Issue とフィードバックは大歓迎です |

**法務** — [LICENSE](LICENSE) (リポジトリの条件) ·
[EULA](legal/EULA.md) ·
[プライバシー](legal/PRIVACY.md) ·
[サードパーティ表示](legal/THIRD_PARTY_NOTICES.md)

法的文書は現在、英語（および韓国語）でのみご用意しています。日本語版はまだありません。
EULA の定めにより、翻訳と英語版が食い違う場合は英語版が優先します。

プロプライエタリソフトウェアであり、現在は EULA のもと**個人利用および社内業務利用は無償**です。
再販売・改変・リバースエンジニアリング・再配布・他製品への組み込みは制限されます。今後の機能や
サービスには別の条件が付くことがあり、その場合は適用前にお知らせします。

アプリの表示言語は **English · 한국어 · 日本語 · 中文** に対応しています。

**商標** — Apache Kafka および Kafka は Apache Software Foundation の商標です。Kaflow Search は
ASF と提携しておらず、ASF による推奨・後援も受けていません。その他の名称および商標は各権利者に
帰属します。

---

<div align="center">

**Kaflow Search** — Kafka メッセージのためのデスクトップ検索エンジン。

[ダウンロード](https://github.com/whsoul/kaflow-search/releases/latest) ·
[Issue](https://github.com/whsoul/kaflow-search/issues) ·
[プライバシー](legal/PRIVACY.md)

Copyright © 2026 Whsoul Tools. Kaflow Search is a product of Whsoul Tools.

</div>
