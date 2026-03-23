# Compatibility Matrix

## 目的

この文書は、broker が各 compatibility mode と transport において、
何を accept し、inspect し、transform し、forward し、cache し、replay
できるかを定義します。

目的は、通常の OSC 互換を保ちつつ、enhanced behavior を偶然ではなく
明示的なものにすることです。

## 解釈の基準

- OSC 1.0 を互換性の基準にする
- type tag string を省略する古い sender は tolerant mode で扱う
- OSC 1.1 系の挙動は additive かつ opt-in とする
- security、discovery、schema、recovery は raw OSC compatibility の上に
  レイヤーとして積む

## Compatibility Mode

### `osc1_0_strict`

使いどころ:

- standards correctness を最優先したい
- advanced handling 前に safely inspectable であることが重要

主な挙動:

- valid な OSC 1.0 message / bundle structure を要求
- 明示的な type tag string を要求
- address pattern は OSC 1.0 のみ
- 未対応 tagged value は transform 対象にしない

### `osc1_0_legacy_tolerant`

使いどころ:

- 古い実装を統合したい
- deep inspection より互換性維持が重要

主な挙動:

- type tag string 省略 message を受理
- address ベース routing を許可
- opaque payload を forwarding と replay 用に保持
- route policy が decoder を明示しない限り argument-aware transform を禁止

### `osc1_1_extended`

使いどころ:

- 双方が意図して extended behavior を使う
- richer pattern、type、metadata が有益

主な挙動:

- OSC 1.0 baseline compatibility を維持
- `//` path traversal wildcard を有効化できる
- richer type support や stream metadata を有効化できる
- それでも enhanced behavior を legacy raw OSC payload に強制しない

## Transport Matrix

| Transport | Status | Notes |
| --- | --- | --- |
| UDP datagram | First-class | default compatibility path |
| TCP size-prefix framing | Supported | stream transport 向け compatibility mode |
| SLIP framing | Supported | 1.1 guidance に沿う additive framing |
| WebSocket / JSON adapter | Additive | adapter path であり raw OSC baseline ではない |
| MQTT adapter | Additive | adapter path であり raw OSC baseline ではない |
| Shared memory IPC | Additive | local acceleration path、必須ではない |

## Address Pattern Matrix

| Feature | `osc1_0_strict` | `osc1_0_legacy_tolerant` | `osc1_1_extended` |
| --- | --- | --- | --- |
| `/foo/bar` literal path | Yes | Yes | Yes |
| `?` single-character wildcard | Yes | Yes | Yes |
| `*` wildcard within part | Yes | Yes | Yes |
| `[]` character set | Yes | Yes | Yes |
| `{foo,bar}` alternation | Yes | Yes | Yes |
| `//` path traversal wildcard | No | No | Optional |

## Message Structure Matrix

| Packet Shape | `osc1_0_strict` | `osc1_0_legacy_tolerant` | `osc1_1_extended` |
| --- | --- | --- | --- |
| type tag 付き well-formed message | Accept | Accept | Accept |
| well-formed bundle | Accept | Accept | Accept |
| type tag なし message | Reject | legacy opaque として Accept | Optional、通常は tolerant と同様 |
| malformed packet | Reject | Reject または quarantine | Reject または quarantine |
| nested bundle | Accept | Accept | Accept |

## Value Support Matrix

broker は少なくとも次の 4 段階を区別します。

- `Accept`
- `Inspect`
- `Transform`
- `Cache`

### 必須 OSC 1.0 Type

| Type | Accept | Inspect | Transform | Cache |
| --- | --- | --- | --- | --- |
| `i` int32 | Yes | Yes | Yes | route policy 次第 |
| `f` float32 | Yes | Yes | Yes | route policy 次第 |
| `s` string | Yes | Yes | Yes | route policy 次第 |
| `b` blob | Yes | Yes | Cautious | 通常は default で no |

### Legacy / Optional / Extended Type

| Type | `osc1_0_strict` | `osc1_0_legacy_tolerant` | `osc1_1_extended` |
| --- | --- | --- | --- |
| `h` int64 | Accept、実装時 inspect | opaque なら address-route only | Accept、inspect |
| `t` timetag argument | Accept、実装時 inspect | safe decode 不可なら opaque | Accept、inspect |
| `d` double | Accept、実装時 inspect | safe decode 不可なら opaque | Accept、inspect |
| `S` symbol | Accept、実装時 inspect | safe decode 不可なら opaque | Accept、inspect |
| `c` char | Accept、実装時 inspect | safe decode 不可なら opaque | Accept、inspect |
| `r` rgba | Accept、実装時 inspect | safe decode 不可なら opaque | Accept、inspect |
| `m` MIDI | Accept、実装時 inspect | safe decode 不可なら opaque | Accept、inspect |
| `T`, `F`, `N`, `I` | 実装済みなら Accept | safe decode 不可なら opaque | Accept、inspect |
| `[` `]` arrays | 実装済みなら Accept | safe decode 不可なら opaque | Accept、inspect |
| unknown tagged value | raw packet intact なら forward、no transform | raw packet intact なら forward | raw packet intact なら forward、route-specific decoder がない限り no transform |

## Capability Matrix

| Capability | `osc1_0_strict` | `osc1_0_legacy_tolerant` | `osc1_1_extended` |
| --- | --- | --- | --- |
| byte-exact forwarding | Yes | Yes | Yes |
| address-based routing | Yes | Yes | Yes |
| argument-aware routing | Yes | Limited | Yes |
| value transform | safely decoded のとき Yes | default では No | safely decoded のとき Yes |
| stateful cache candidate | route policy 次第で Yes | opaque legacy payload は default で No | route policy 次第で Yes |
| replay from raw bytes | Yes | Yes | Yes |
| rehydrate from decoded state | cacheable のとき Yes | default では No | cacheable のとき Yes |

## Security Overlay Matrix

| Behavior | raw OSC compatibility route | secure overlay route |
| --- | --- | --- |
| authentication required | No | Yes |
| authorization enforced | optional broker policy | Yes |
| legacy peer 向け payload 変更 | No | No、broker が secure envelope を終端してから forwarding |
| verification error 時の失敗 | route drop policy | fail closed |

## Discovery And Metadata Matrix

| Feature | compatibility baseline | additive support |
| --- | --- | --- |
| static manual endpoint config | Yes | Yes |
| DNS-SD / mDNS discovery | Optional | Yes |
| stream metadata publication | Optional | Yes |
| service URI metadata | Optional | Yes |

## Replay And Recovery Matrix

| Behavior | `osc1_0_strict` | `osc1_0_legacy_tolerant` | `osc1_1_extended` |
| --- | --- | --- | --- |
| byte-exact replay | Yes | Yes | Yes |
| cache からの state rehydrate | decoded state があれば Yes | opaque legacy payload は通常 No | decoded state があれば Yes |
| late joiner sync | route policy 次第で Yes | route decoder または raw resend policy があるときのみ | route policy 次第で Yes |

## 例

### Example A: Standard UE5 Control Message

Packet:

- address `/ue5/camera/fov`
- type tags `,f`
- float argument

結果:

- 3 mode すべてで accept
- inspectable
- transformable
- route policy が許せば cacheable

### Example B: type tag のない legacy packet

Packet:

- address `/legacy/position`
- type tag string なし

結果:

- `osc1_0_strict` では reject
- `osc1_0_legacy_tolerant` では `LegacyUntypedMessage` として accept
- forwardable / replayable
- route decoder がない限り argument transform は不可

### Example C: Extended Path Traversal

Pattern:

- `//spherical`

結果:

- `osc1_0_strict` では unavailable
- `osc1_0_legacy_tolerant` でも unavailable
- `osc1_1_extended` で optional

## 非交渉の互換性ルール

- UDP raw OSC 1.0 を第一級扱いにする
- type-tag-less legacy packet の許容は tolerant path に限定する
- enhanced behavior が raw OSC expectation を黙って書き換えてはならない
- unknown tagged payload は forwardable でも transformable とは限らない
- security overlay は downstream の legacy interoperability を壊してはならない
