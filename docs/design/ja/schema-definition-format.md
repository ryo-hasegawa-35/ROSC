# Schema Definition Format

## 目的

この文書は、validation、tooling、code generation に使う optional な typed OSC
schema の semantic structure を定義します。

schema system は意図的に optional です。OSC を成功させた ad hoc flexibility を
壊さずに reliability を上げることが目的です。

## 設計目標

- optional であり、raw OSC に mandatory ではない
- bureaucracy より validation にまず価値を出す
- language-neutral
- generation-friendly
- route-level / namespace-level の両方で使える

## Scope

schema format が記述できるべきもの:

- namespace
- address
- message argument
- unit
- constraint
- documentation
- versioning
- compatibility expectation

## Schema Layer

### Layer 1: Namespace Description

定義するもの:

- namespace owner
- namespace purpose
- compatibility mode expectation
- security scope hint

### Layer 2: Address Definition

定義するもの:

- concrete address または pattern
- message role
- argument sequence
- caching suitability
- recovery suitability

### Layer 3: Argument Definition

定義するもの:

- type
- optionality
- unit
- range constraint
- enum または semantic meaning

### Layer 4: Tooling Metadata

定義するもの:

- documentation text
- generation hint
- deprecation marker
- migration note

## Core Entity

想定 top-level entity:

- `schema`
- `namespace`
- `message`
- `argument`
- `enum`
- `constraint`
- `profile`

## Type Model

schema type model は supported broker value model に写るべきです。

baseline type:

- `int32`
- `float32`
- `string`
- `blob`

extended type:

- `int64`
- `double64`
- `timetag`
- `symbol`
- `char`
- `rgba`
- `midi4`
- `bool_literal`
- `nil`
- `impulse`
- `array`

## Constraint Model

schema が支えるべき constraint:

- minimum / maximum
- allowed enum value
- 必要に応じた regex-like textual restriction
- array length bound
- key extraction hint
- idempotent-state hint

## Semantic Hint

recovery と operations を支えるため、schema は次を表現できるべきです。

- message が state-like か trigger-like か
- late joiner rehydrate が safe か
- caching 推奨か
- transform が safe か

## Versioning

各 schema が宣言すべきもの:

- schema format version
- domain version
- compatibility statement

推奨 compatibility state:

- backward compatible
- additive change
- breaking change

## Generation Target

schema が generation hint を持つべき対象:

- Rust value binding
- C ABI descriptor
- C++ integration helper
- Python integration helper
- validation manifest

## Raw OSC との関係

critical rule:

- schema は intended meaning を記述する
- schema は raw OSC packet validity 自体を再定義しない

つまり:

- valid OSC でも schema non-conformant でありうる
- schema validation はそれを明示的に報告する
- schema use は route、namespace、tool 単位の opt-in にする

## Example Semantic Shape

message definition は少なくとも次に答えられるべきです。

- これはどの address か
- 各 argument は何を意味するか
- どの unit を期待するか
- どの range が safe か
- これは state か trigger か
- cache できるか
- rehydrate に使えるか

## Documentation Expectation

schema entry は人間が読んでも有用であるべきです。

最低限ほしい human-oriented field:

- display name
- description
- example
- 必要に応じた operational note

## Validation Level

schema tool が少なくとも持つべきもの:

- lint
- strict validation
- migration guidance

## Validation Cost Policy

schema validation は、すべての route に同じ深さで掛ける前提にすべきでは
ありません。

推奨 validation depth:

- `off`: data plane では schema validation を行わない
- `shape_only`: arity と大まかな型ファミリだけを安価に確認する
- `typed`: schema どおりの argument type を確認する
- `strict`: 型に加えて range、enum、semantic constraint まで確認する

推奨 default:

- critical control route は `typed` または `strict`
- moderate-rate な stateful route は `typed`
- high-rate sensor / telemetry route は、明示的に許可しない限り `off` または
  `shape_only`

この tradeoff は routing plan、benchmark plan、operator UI のすべてで
見えるようにすべきです。

## 非交渉の不変条件

- schema は optional のまま保つ
- schema が raw OSC compatibility を弱めてはならない
- schema versioning は明示的であること
- generated code は schema intent を反映し、追加 semantics を捏造しない
- schema は ceremony を増やすより ambiguity を減らすべき
