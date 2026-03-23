# Config Validation And Migration Note

## 目的

この文書は、configuration を apply 前にどう validation するか、そして
configuration version を将来どう migration していくかを定義します。

## 設計目標

- bad config を live routing に到達する前に拒否する
- hot reload 中の operator confidence を保つ
- upgrade を explicit かつ auditable にする
- last known-good config への rollback を支える

## Validation Stage

### Stage 1: Syntax Validation

確認内容:

- file が parse 可能か
- 想定 top-level structure を持つか
- supported schema version field があるか

### Stage 2: Schema Validation

確認内容:

- required field があるか
- enum value が妥当か
- field type が妥当か
- duplicate ID を拒否できるか

### Stage 3: Semantic Validation

確認内容:

- mode と pattern の組み合わせが妥当か
- destination reference が解決できるか
- cache と recovery policy の組み合わせが安全か
- security policy が内部整合しているか
- build profile に対して plugin reference が妥当か

### Stage 4: Runtime Validation

確認内容:

- 必要な port が利用可能か
- adapter が存在し有効か
- 危険な live transition が flag されるか

## Validation Output

validation は少なくとも次に分類すべきです。

- error
- warning
- advisory

error は apply を block します。
warning は policy に応じて明示確認を要求してよいです。

## Migration Model

各 config が持つべきもの:

- schema version
- compatibility profile version
- optional な migration history

## Migration Type

### Automatic Safe Migration

向いている場合:

- field rename が機械的
- default insertion が安全

### Assisted Migration

向いている場合:

- semantics が変わった
- operator choice が必要

### Manual Migration

向いている場合:

- 自動化すると危険に behavior が変わりうる

## Last-Known-Good Policy

保持すべきもの:

- 最後に apply された good config
- validation report
- apply timestamp
- 必要なら operator identity

## Hot Reload Rule

- cutover 前に fully validate する
- 可能な限り atomically apply する
- apply 失敗時は last-known-good を維持する
- 危険な change は operator に config diff を見せる

## Compatibility Profile

config validation は compatibility intent を理解すべきです。

- strict routing profile
- legacy tolerant profile
- extended feature profile
- secure profile
- safe mode profile

## Auditability

最低限記録すべきもの:

- 誰が config を apply したか
- 何が変わったか
- validation result
- rollback event

## 非交渉の不変条件

- config apply が partial かつ silent であってはならない
- migration が visibility なしに behavior を書き換えてはならない
- rollback path を残す
- validation は syntax だけでなく semantics を含む
