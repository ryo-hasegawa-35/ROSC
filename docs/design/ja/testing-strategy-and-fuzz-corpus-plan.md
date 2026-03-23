# Testing Strategy And Fuzz Corpus Plan

## 目的

この文書は、implementation 開始後にどう testing を進めるべきか、また fuzz
target をどう整理するべきかを定義します。

目的は correctness だけではありません。stress、malformed input、
long-running operation に対する confidence も含みます。

## Testing Priorities

- protocol correctness
- compatibility correctness
- failure containment
- recovery correctness
- performance predictability
- security behavior

## Test Layer

### Layer 1: Unit Test

用途:

- parser の部品
- encoding
- route matching
- value normalization
- config validation

### Layer 2: Property Test

用途:

- encode / decode roundtrip property
- 必要な ordering guarantee
- compatibility mode 間の invariant

### Layer 3: Integration Test

用途:

- ingress-to-egress routing
- route policy behavior
- adapter interaction
- cache / recovery behavior

### Layer 4: Fault Injection Test

用途:

- slow destination
- malformed traffic storm
- plugin timeout
- adapter disconnect
- degraded mode transition

### Layer 5: Soak Test

用途:

- long-running reliability
- memory growth detection
- queue stability
- repeated disruption 後の recovery

## Fuzzing Strategy

fuzzing は byte-level だけでなく semantic-level の failure surface も狙うべき
です。

## Fuzz Corpus Family

### Packet Parsing Corpus

含めるもの:

- valid OSC 1.0 message
- valid bundle
- nested bundle
- truncated packet
- misaligned padding
- malformed type tag string

### Legacy Compatibility Corpus

含めるもの:

- missing type tag packet
- ambiguous legacy payload
- opaque だが forwardable な edge case

### Extended Type Corpus

含めるもの:

- optional tagged value
- array
- unknown tag
- typed / unsupported content の混在

### Framing Corpus

含めるもの:

- size-prefixed stream edge case
- SLIP framing error
- broken packet boundary sequence

### Config Corpus

含めるもの:

- duplicate route ID
- invalid compatibility combination
- unsafe cache / recovery combination
- invalid profile combination

### Security Corpus

含めるもの:

- malformed secure envelope field
- expired token
- mismatched scope
- replay-like sequence

## Golden Reference Material

project は次の golden vector を維持すべきです。

- specification 由来の OSC 1.0 example
- 選定した 1.1-oriented compatibility example
- 既知の legacy tolerant case
- cache / rehydrate policy case

## Regression Policy

実際に起きた bug は必ず次の 1 つ以上を追加すべきです。

- unit または integration regression test
- fuzz corpus seed
- relevant なら golden replay または capture artifact

## Environment Matrix

最終的に testing がカバーすべきもの:

- Windows
- macOS
- Linux
- behavior が異なる複数 release profile

## 非交渉の不変条件

- compatibility bug には permanent regression test を作る
- malformed input を niche concern 扱いしない
- recovery behavior は code shape から推測せず behavior として test する
- fuzzing は packet byte だけでなく config / security surface も含む
