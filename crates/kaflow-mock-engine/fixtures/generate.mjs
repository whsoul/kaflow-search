// Demo fixture generator for the mock engine. Deterministic: the seed is fixed, so
//
//   node crates/kaflow-mock-engine/fixtures/generate.mjs
//
// re-running it produces the same output.
//
// Writes synthetic messages over default.json: high-cardinality values across a mix of
// field types. Offsets are omitted (the loader assigns them per partition), and timestamps
// increase together with the offset within each partition.

import { writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const __dir = dirname(fileURLToPath(import.meta.url));

// ── seeded RNG (mulberry32) ───────────────────────────────────────────────
function mulberry32(seed) {
  return function () {
    seed |= 0;
    seed = (seed + 0x6d2b79f5) | 0;
    let t = Math.imul(seed ^ (seed >>> 15), 1 | seed);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}
const rng = mulberry32(20260626);
const pick = (a) => a[Math.floor(rng() * a.length)];
const ri = (lo, hi) => lo + Math.floor(rng() * (hi - lo + 1));
const pad = (n, w) => String(n).padStart(w, "0");
const chance = (p) => rng() < p;
const hex = (n) => {
  let s = "";
  for (let i = 0; i < n; i++) s += "0123456789abcdef"[Math.floor(rng() * 16)];
  return s;
};

const BASE = 1704067200000; // 2024-01-01T00:00:00Z
const WEEK = 7 * 24 * 3600 * 1000;

// ── pools ─────────────────────────────────────────────────────────────────
const PRODUCTS = [
  "무선 이어폰", "충전 케이블", "스마트폰 케이스", "기계식 키보드", "마우스 패드",
  "USB 허브", "무선 마우스", "노트북 스탠드", "보조 배터리", "27인치 모니터",
  "젤리 케이스", "블루투스 스피커", "무선 충전기", "태블릿", "강화유리 필름",
  "키캡 세트", "웹캠", "마이크", "외장 SSD", "그래픽 태블릿",
  "게이밍 헤드셋", "USB-C 독", "HDMI 케이블", "스마트워치", "피트니스 밴드",
  "전동 칫솔", "커피 그라인더", "텀블러", "백팩", "노트북 파우치",
  "데스크 매트", "모니터 암", "독서대", "LED 스탠드", "공기청정기",
  "가습기", "미니 선풍기", "전기 포트", "에어프라이어", "로봇청소기",
];
const STATUS = ["PENDING", "PAID", "SHIPPED", "DELIVERED", "CANCELLED", "REFUNDED"];
const REGIONS = ["KR", "US", "JP", "DE", "GB", "FR", "SG", "AU", "CA", "IN"];
const CURRENCIES = ["KRW", "USD", "JPY", "EUR", "GBP"];
const PAY = ["card", "bank_transfer", "paypal", "kakaopay", "applepay", "naverpay"];
const CHANNELS = ["web", "ios", "android", "kiosk"];

const PLANS = ["free", "pro", "team", "enterprise"];
const COUNTRIES = ["KR", "US", "JP", "DE", "GB", "FR", "SG", "AU", "CA", "IN", "BR", "MX", "ES", "IT", "NL"];
const DOMAINS = ["example.com", "gmail.com", "naver.com", "outlook.com", "company.io", "acme.co", "startup.dev"];
const FIRST = ["alice", "bob", "carol", "dave", "erin", "frank", "grace", "heidi", "ivan", "judy", "mallory", "oscar", "peggy", "trent", "victor", "wendy", "minji", "jihoon", "yuna", "seojun"];
const REFERRERS = ["google", "twitter", "friend_invite", "paid_ad", "organic", "github", "producthunt", "newsletter"];
const DEVICES = ["web", "ios", "android"];

const LEVELS = ["DEBUG", "INFO", "INFO", "INFO", "WARN", "WARN", "ERROR"]; // weighted
const SERVICES = ["auth-service", "order-service", "index-service", "payment-service", "search-service", "notification-service", "gateway", "user-service", "billing-service", "kafka-consumer", "scheduler", "media-service"];
const LOG_TEMPLATES = [
  (c) => `user login succeeded for ${c.user}`,
  (c) => `user logout for ${c.user}`,
  (c) => `token validation failed invalid signature for ${c.user}`,
  (c) => `payment gateway slow response detected latency ${c.lat}ms`,
  (c) => `order created successfully ${c.order}`,
  (c) => `refund processing failed insufficient balance order ${c.order}`,
  (c) => `background indexing completed ${c.cnt} messages`,
  (c) => `index compaction took longer than expected ${c.lat}ms`,
  (c) => `cache miss for key ${c.key} fallback to db`,
  (c) => `rate limit exceeded for client ${c.ip}`,
  (c) => `connection pool exhausted retrying in ${c.lat}ms`,
  (c) => `scheduled job ${c.job} finished with status ${c.st}`,
  (c) => `email notification sent to ${c.user}`,
  (c) => `database query timeout on table ${c.tbl}`,
];

function tsFor(i, n) {
  // Globally increasing with jitter, so timestamps stay ordered within every partition too.
  return BASE + Math.floor((i / n) * WEEK) + ri(0, 30000);
}

function buildOrders(n, partitions) {
  const customers = Array.from({ length: 180 }, (_, i) => `C-${pad(i + 1, 4)}`);
  const msgs = [];
  for (let i = 0; i < n; i++) {
    const itemCount = ri(1, 4);
    const items = Array.from({ length: itemCount }, () => ({
      name: pick(PRODUCTS),
      qty: ri(1, 5),
      price: ri(1000, 200000),
    }));
    const orderId = `ORD-${pad(i + 1, 6)}`;
    msgs.push({
      partition: i % partitions,
      ts_millis: tsFor(i, n),
      key: orderId,
      value: {
        orderId,
        customerId: pick(customers),
        amount: items.reduce((s, it) => s + it.qty * it.price, 0),
        currency: pick(CURRENCIES),
        status: pick(STATUS),
        region: pick(REGIONS),
        paymentMethod: pick(PAY),
        channel: pick(CHANNELS),
        items,
      },
    });
  }
  return { name: "orders-events", partitions, key_deserializer: "plain", value_deserializer: "json", messages: msgs };
}

function buildSignups(n, partitions) {
  const msgs = [];
  for (let i = 0; i < n; i++) {
    const userId = `U-${pad(i + 1, 5)}`;
    const name = `${pick(FIRST)}${ri(1, 9999)}`;
    msgs.push({
      partition: i % partitions,
      ts_millis: tsFor(i, n),
      key: userId,
      value: {
        userId,
        email: `${name}@${pick(DOMAINS)}`,
        plan: pick(PLANS),
        country: pick(COUNTRIES),
        referrer: pick(REFERRERS),
        device: pick(DEVICES),
        age: ri(18, 70),
        marketingOptIn: chance(0.6),
      },
    });
  }
  return { name: "user-signups", partitions, key_deserializer: "plain", value_deserializer: "json", messages: msgs };
}

function buildLogs(n, partitions) {
  const msgs = [];
  for (let i = 0; i < n; i++) {
    const service = pick(SERVICES);
    const ctx = {
      user: `${pick(FIRST)}${ri(1, 999)}`,
      order: `ORD-${pad(ri(1, 99999), 6)}`,
      lat: ri(5, 4200),
      cnt: ri(10, 5000),
      key: `cache:${hex(8)}`,
      ip: `${ri(1, 255)}.${ri(0, 255)}.${ri(0, 255)}.${ri(1, 255)}`,
      job: `job-${pick(["cleanup", "resync", "rollup", "ilm", "compaction"])}`,
      st: pick(["ok", "failed", "skipped"]),
      tbl: pick(["orders", "users", "sessions", "events", "index_meta"]),
    };
    msgs.push({
      partition: i % partitions,
      ts_millis: tsFor(i, n),
      key: service,
      value: {
        level: pick(LEVELS),
        service,
        traceId: hex(16),
        message: pick(LOG_TEMPLATES)(ctx),
        latencyMs: ctx.lat,
        statusCode: pick([200, 200, 201, 400, 401, 404, 500, 503]),
      },
    });
  }
  return { name: "app-logs", partitions, key_deserializer: "plain", value_deserializer: "json", messages: msgs };
}

// ── Multilingual review topic (English / Japanese / Chinese / Korean) ────────────────
const EN = {
  code: "en",
  products: ["Wireless Earbuds", "Mechanical Keyboard", "Gaming Mouse", "USB-C Hub", "Portable SSD", "Bluetooth Speaker", "Smart Watch", "Webcam", "Monitor Stand", "Noise Cancelling Headphones", "Power Bank", "Tablet"],
  pos: ["excellent", "amazing", "great value for money", "highly recommended", "works perfectly", "absolutely love it", "fast shipping"],
  neg: ["disappointing", "stopped working after a week", "poor build quality", "not as described", "arrived damaged", "too expensive for what it is", "battery drains too fast"],
  aspects: ["battery life", "build quality", "sound quality", "comfort", "price", "design", "connectivity"],
  categories: ["Electronics", "Audio", "Computer Accessories", "Wearables"],
  authors: ["James", "Olivia", "Liam", "Emma", "Noah", "Ava", "William", "Sophia"],
  title: (p, s) => `${p} — ${s}`,
  body: (p, a, s, pos) => `I bought this ${p} last month. The ${a} is ${pos ? "really good" : "below average"} and overall it was ${s}. ${pos ? "Would buy again." : "I would not recommend it."}`,
};
const JA = {
  code: "ja",
  products: ["ワイヤレスイヤホン", "メカニカルキーボード", "ゲーミングマウス", "USBハブ", "ポータブルSSD", "Bluetoothスピーカー", "スマートウォッチ", "ウェブカメラ", "モニタースタンド", "ノイズキャンセリングヘッドホン", "モバイルバッテリー", "タブレット"],
  pos: ["最高でした", "とても良いです", "コスパが高い", "おすすめです", "完璧に動作します", "とても気に入りました", "発送が速かった"],
  neg: ["がっかりしました", "一週間で壊れました", "作りが安っぽい", "説明と違います", "破損して届きました", "値段の割に良くない", "バッテリーの減りが早い"],
  aspects: ["バッテリー持ち", "作りの良さ", "音質", "着け心地", "価格", "デザイン", "接続の安定性"],
  categories: ["家電", "オーディオ", "パソコン周辺機器", "ウェアラブル"],
  authors: ["田中", "佐藤", "鈴木", "山田", "伊藤", "渡辺", "高橋"],
  title: (p, s) => `${p}：${s}`,
  body: (p, a, s, pos) => `先月この${p}を購入しました。${a}は${pos ? "とても良く" : "いまひとつで"}、全体的に${s}。${pos ? "また買いたいです。" : "あまりおすすめしません。"}`,
};
const ZH = {
  code: "zh",
  products: ["无线耳机", "机械键盘", "游戏鼠标", "USB集线器", "移动固态硬盘", "蓝牙音箱", "智能手表", "网络摄像头", "显示器支架", "降噪耳机", "充电宝", "平板电脑"],
  pos: ["非常好", "物超所值", "强烈推荐", "运行完美", "非常喜欢", "发货很快", "质量很棒"],
  neg: ["很失望", "用了一周就坏了", "做工很差", "与描述不符", "到货时已损坏", "性价比不高", "电池耗电太快"],
  aspects: ["电池续航", "做工", "音质", "舒适度", "价格", "设计", "连接稳定性"],
  categories: ["电子产品", "音频", "电脑配件", "可穿戴设备"],
  authors: ["张伟", "王芳", "李娜", "刘洋", "陈杰", "杨静", "赵强"],
  title: (p, s) => `${p}：${s}`,
  body: (p, a, s, pos) => `上个月买了这款${p}。${a}${pos ? "很不错" : "比较一般"}，总体来说${s}。${pos ? "会再次购买。" : "不太推荐。"}`,
};
const KO = {
  code: "ko",
  products: ["무선 이어폰", "기계식 키보드", "게이밍 마우스", "USB 허브", "외장 SSD", "블루투스 스피커", "스마트워치", "웹캠", "모니터 받침대", "노이즈 캔슬링 헤드폰", "보조 배터리", "태블릿"],
  pos: ["아주 만족합니다", "정말 좋아요", "가성비가 훌륭합니다", "강력 추천합니다", "완벽하게 작동해요", "아주 마음에 듭니다", "배송이 빨랐어요"],
  neg: ["많이 실망했어요", "일주일 만에 고장났어요", "마감이 별로예요", "설명과 다릅니다", "파손되어 도착했어요", "가격 대비 별로예요", "배터리가 너무 빨리 닳아요"],
  aspects: ["배터리 수명", "마감 품질", "음질", "착용감", "가격", "디자인", "연결 안정성"],
  categories: ["전자제품", "오디오", "컴퓨터 주변기기", "웨어러블"],
  authors: ["김민수", "이지훈", "박서연", "최유나", "정현우", "한지민"],
  title: (p, s) => `${p} - ${s}`,
  body: (p, a, s, pos) => `지난달에 이 ${p}을(를) 구매했습니다. ${a}이(가) ${pos ? "정말 좋고" : "기대 이하였고"} 전반적으로 ${s}. ${pos ? "또 구매할 의향이 있어요." : "추천하지 않습니다."}`,
};

function buildReviews(name, P, n, partitions) {
  const msgs = [];
  for (let i = 0; i < n; i++) {
    const rating = ri(1, 5);
    const positive = rating >= 4;
    const sent = positive ? pick(P.pos) : pick(P.neg);
    const aspect = pick(P.aspects);
    const product = pick(P.products);
    const reviewId = `RV-${P.code.toUpperCase()}-${pad(i + 1, 5)}`;
    msgs.push({
      partition: i % partitions,
      ts_millis: tsFor(i, n),
      key: reviewId,
      value: {
        reviewId,
        language: P.code,
        product,
        category: pick(P.categories),
        rating,
        title: P.title(product, sent),
        body: P.body(product, aspect, sent, positive),
        author: pick(P.authors),
        verifiedPurchase: chance(0.7),
        helpfulVotes: ri(0, 240),
      },
    });
  }
  return { name, partitions, key_deserializer: "plain", value_deserializer: "json", messages: msgs };
}

const root = {
  _comment:
    "GENERATED by generate.mjs (seed 20260626) — edit the script and re-run it rather than this file. High-cardinality multilingual demo data.",
  topics: [
    buildOrders(420, 4),
    buildSignups(300, 3),
    buildLogs(480, 4),
    buildReviews("reviews-en", EN, 180, 3),
    buildReviews("reviews-ja", JA, 180, 3),
    buildReviews("reviews-zh", ZH, 180, 3),
    buildReviews("reviews-ko", KO, 180, 3),
  ],
};

const out = join(__dir, "default.json");
writeFileSync(out, JSON.stringify(root, null, 2) + "\n");
const total = root.topics.reduce((s, t) => s + t.messages.length, 0);
console.log(`wrote ${out}: ${root.topics.length} topics, ${total} messages`);
for (const t of root.topics) console.log(`  ${t.name}: ${t.messages.length} msgs / ${t.partitions} partitions`);
