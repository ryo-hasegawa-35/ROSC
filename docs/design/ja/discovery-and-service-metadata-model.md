# Discovery And Service Metadata Model

## 目的

この文書は、broker、adapter、endpoint、service をどう discovery し、
どう記述するかを定義します。manual control や backward compatibility を
弱めないことが前提です。

discovery は convenience layer です。operator が理由を説明できない hidden
control plane になってはなりません。

## 設計目標

- manual configuration を baseline として残す
- discovery を additive に保つ
- service を transport-neutral に記述する
- discovered fact と trusted fact を区別する
- discovery result を可視かつ audit 可能にする

## Discovery 原則

- discovery は configuration を提案してよいが、黙って作成してはならない
- discovery freshness は明示する
- identity と trust は別概念として扱う
- discovery は operator を助けるべきで、置き換えるべきではない
- metadata を publish しない legacy OSC endpoint も first-class として扱う

## Discovery Mode

### Manual Only

向いている場合:

- deterministic operation を最優先したい
- 環境が小さいまたは安定している
- discovery traffic を避けたい

### Passive Discovery

向いている場合:

- endpoint 自身が announce する
- broker は active probe せず catalog 化したい

例:

- DNS-SD / mDNS advertisement
- broker peer advertisement

### Active Discovery

向いている場合:

- broker が意図的に環境を probe する
- operator が guided setup を欲している

例:

- mDNS browse query
- adapter-specific endpoint enumeration

### Registered Discovery

向いている場合:

- service metadata が registry または explicit operator input で与えられる
- 環境がより大きいか、より統制されている

## Discovery Entity

model は次の entity を区別すべきです。

### Broker

runtime instance 自体。

### Adapter

broker が持つ transport / protocol-specific boundary。

### Service

network または local environment に公開される logical endpoint capability。

例:

- OSC UDP receiver
- WebSocket control service
- MQTT bridge

### Endpoint

host、port、topic、path、session、channel などの concrete で addressable な
target。

### Capability

transport type、framing、supported pattern、security requirement など、
service が持つ declared property。

## Service Metadata Model

各 discovered service は transport-neutral な record として表現できるべきです。

想定 field:

- `service_id`
- `service_kind`
- 必要なら `broker_id`
- 必要なら `adapter_id`
- `display_name`
- `protocol_family`
- `transport`
- `framing`
- `version`
- `endpoint_refs`
- `capabilities`
- `scope`
- `security_mode`
- `metadata_source`
- `first_seen_at`
- `last_seen_at`
- `ttl`

## Endpoint Metadata Model

想定 endpoint field:

- `endpoint_id`
- `host`
- `port`
- `path_or_topic`
- `interface`
- `locality`
- `session_requirement`
- `identity_requirement`

## Capability Advertisement

service metadata は少なくとも次を記述できるべきです。

- supported protocol family
- supported framing
- supported compatibility mode
- security expectation
- discovery が passive か active か
- 使用前に operator approval が必要か

## Trust Level

discovery result は trust classification を持つべきです。

想定 level:

- `Observed`
- `Claimed`
- `Verified`
- `OperatorApproved`

解釈:

- `Observed`
  - network 上で見えたが trust はしていない
- `Claimed`
  - self-advertised capability
- `Verified`
  - trusted mechanism により identity または capability を確認
- `OperatorApproved`
  - operator が明示的に受け入れた

## Freshness と Expiry

discovery data は永遠に残してはなりません。

推奨ルール:

- すべての discovered record に `last_seen_at` を持つ
- TTL 付き record は expiry を明示する
- expiry した record を active configuration として黙って再利用しない
- stale record は operator history としては残してよい

## Discovery と Routing

discovery 単独で route behavior を作ってはなりません。

許される相互作用:

- destination candidate を提案する
- UI の label や metadata を補う
- route configuration 作成を助ける

禁止すべき相互作用:

- discovered destination を黙って live route に接続する
- explicit security policy を bypass する
- compatibility mode を暗黙に決める

## Discovery と Security

discovery と trust は分離されるべきです。

ルール:

- discovered service identity は verified identity とは別
- security-sensitive route は explicit approval または verified trust を要求
- anonymous legacy discovery も visibility 用としては有用

## Discovery Failure Mode

システムは次に耐えるべきです。

- discovery unavailable
- noisy discovery environment
- stale advertisement
- conflicting advertisement
- insecure environment での spoofed advertisement

failure handling:

- manual configuration は常に使える
- stale / conflicting discovery は visible に出す
- secure deployment profile は untrusted discovery result を無視できる

## Operator Experience

operator ができるべきこと:

- discovered service を browse する
- trust と freshness を確認する
- discovered service を approve / reject する
- discovery result を explicit configuration へ変換する
- discovered service の disappearance を把握する

## Inter-Broker Discovery

broker は次を advertise してよいです。

- broker ID
- supported profile
- replication / federation capability
- health visibility endpoint

ただし broker discovery だけで federation を自動作成してはなりません。

## 非交渉の不変条件

- discovery は manual configuration を置き換えない
- stale discovery は stale として見える
- visibility だけで discovery trust を仮定しない
- discovery が live route を黙って変えない
- discovery metadata のない legacy OSC も first-class のまま
