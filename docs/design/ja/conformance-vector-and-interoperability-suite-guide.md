# Conformance Vector And Interoperability Suite Guide

## 目的

この文書は、implementation 開始後に conformance vector と interoperability
suite をどう維持するかを定義します。

目的は、互換性の主張を記憶や自信ではなく、再現可能な証拠で守ることです。

## 設計目標

- compatibility claim を testable にする
- protocol correctness と product behavior を区別する
- regression evidence を durable に保つ
- specification conformance と field interoperability の両方を支える

## 2 つの異なる検証系

### Conformance Vector

用途:

- deterministic な reference case
- packet / framing example
- expected parser / encoder behavior
- compatibility mode behavior

### Interoperability Suite

用途:

- real tool または reference implementation との通信確認
- transport behavior check
- actual workflow に対する migration confidence

## Conformance Vector Category

### OSC 1.0 Canonical Vector

含むべきもの:

- standard message
- standard bundle
- padding example
- big-endian numeric example
- address pattern example

### Legacy Tolerant Vector

含むべきもの:

- missing type tag example
- forwardable opaque case
- inspection / transform limitation の期待値

### Extended Compatibility Vector

含むべきもの:

- `//` wildcard behavior
- extended type acceptance behavior
- framing edge case

### Fault Behavior Vector

含むべきもの:

- malformed packet rejection
- unknown type handling
- route drop behavior expectation
- cache / replay safety expectation

## Vector Structure

各 vector が記述すべきもの:

- identifier
- purpose
- input bytes または source artifact
- compatibility mode
- expected parse result
- expected routing eligibility
- expected transform eligibility
- expected cache / replay eligibility

## Interoperability Target

suite が最終的にカバーすべきもの:

- UE5 側 OSC workflow
- TouchDesigner 側 OSC workflow
- adapter 経由の browser control path
- 少なくとも 1 つの MQTT path
- 少なくとも 1 つの secure overlay path

## Interoperability Scenario Shape

各 scenario が定義すべきもの:

- participating component
- transport path
- compatibility mode
- expected success criteria
- known limitation

## Golden Output Policy

stable conformance case では次を保存すべきです。

- input artifact
- expected normalized interpretation
- expected output または state

golden artifact は人間にも機械にも review できる形であるべきです。

## Field Regression Policy

実際の interoperability bug を直したら、必ず次のいずれかを追加すべきです。

- deterministic なら conformance vector
- tool interaction を含むなら interoperability scenario
- timing / ordering が重要なら replay artifact

## Versioning

conformance / interoperability suite が versioning すべきもの:

- vector schema version
- suite version
- 必要なら associated broker design revision

## Reporting

suite が最低限報告すべきもの:

- pass / fail
- environment summary
- used profile
- used compatibility mode
- exercised artifact identifier

## 非交渉の不変条件

- conformance vector は regression comparison に使える程度に stable であること
- interoperability claim は concrete scenario に結び付けること
- field bug は suite を強くすること
- product docs の compatibility language は suite と対応していること
