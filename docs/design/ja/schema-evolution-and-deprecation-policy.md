# Schema Evolution And Deprecation Policy

## 目的

この文書は、optional schema が時間とともにどう進化し、どう deprecation を
扱うべきかを定義します。generated code や live deployment を silent に壊さない
ことが前提です。

## 設計目標

- schema versioning を明示的に保つ
- additive change と breaking change を区別する
- operator / integrator に明確な migration window を与える
- silent semantic drift を避ける

## Evolution 原則

- schema は optional のまま
- version change は visible であること
- deprecation は単なる warning field ではなく communication tool
- generated code は version intent を反映する

## Change Class

### Additive Change

例:

- new optional address
- explicit compatibility note 付きの new optional argument
- consumer が耐えられる new enum value

### Behavioral Change

例:

- unit interpretation の変更
- state / trigger の再解釈
- caching / recovery safety の再解釈

これは通常の additive change より慎重に扱うべきです。

### Breaking Change

例:

- address removal
- argument order change
- type change
- old sender を無効にする required constraint の強化

## Deprecation State

想定 state:

- `active`
- `deprecated`
- `scheduled_removal`
- `removed`

各 deprecated element が持つべきもの:

- since version
- replacement guidance
- 分かるなら removal target

## Versioning Policy

各 schema package が宣言すべきもの:

- format version
- schema version
- compatibility statement

推奨 semantic policy:

- additive change は minor version を上げる
- breaking change は major version を上げる
- documentation only は patch version でよい

## Generated Code への影響

schema change は generated binding に何を要求するか明記すべきです。

- new optional field の追加
- deprecation mark の付与
- incompatible target への generation refusal
- migration hint の提供

## Operational Migration Rule

schema change 時に system が答えられるべきこと:

- old sender はまだ動けるか
- old receiver はまだ動けるか
- route review が必要か
- recovery / cache policy の review が必要か

## Validation Policy

schema-aware tooling が出すべきもの:

- incompatible schema use
- deprecated element use
- migration recommendation

## Documentation Expectation

deprecation または breaking change は少なくとも次を文書化すべきです。

- 何が変わったか
- なぜ変わったか
- 何へ置き換えるべきか
- transition 中に残る risk は何か

## 非交渉の不変条件

- schema evolution は silent であってはならない
- deprecation metadata は machine-readable であること
- breaking change は明確に mark すること
- optional schema tooling が compatibility を偽装してはならないこと
