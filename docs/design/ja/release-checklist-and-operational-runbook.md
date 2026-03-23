# Release Checklist And Operational Runbook

## 目的

この文書は、将来の production profile に対する minimum な release checklist
と operational runbook の形を定義します。

目的は、release を単なる build artifact として扱わないことです。
infrastructure software にとって release quality とは、rollout clarity、
rollback clarity、incident readiness も含みます。

## Release Checklist

## Section A: Build Integrity

確認事項:

- correct profile を build した
- version metadata が入っている
- release note を用意した
- 関連 design document が current である

## Section B: Configuration Safety

確認事項:

- config schema version を把握している
- migration を review した
- last-known-good rollback path がある
- risky config delta を把握している

## Section C: Compatibility Safety

確認事項:

- その profile の compatibility mode が文書化されている
- legacy tolerant behavior が有効なら意図的である
- release note に compatibility risk を正直に書いている

## Section D: Operational Safety

確認事項:

- safe mode path が分かっている
- operator-facing warning が文書化されている
- recovery action が使える
- diagnostics level が profile expectation と一致している

## Section E: Security Safety

必要に応じて確認:

- trust default が文書化されている
- secure overlay が profile に適切
- audit expectation が文書化されている
- insecure fallback behavior を理解している

## Section F: Verification

確認事項:

- relevant な benchmark または regression evidence がある
- profile-specific test expectation を満たした
- known limitation を正直に書いた

## Operational Runbook

runbook が operator に答えるべきこと:

- この profile をどう安全に起動するか
- 健全かどうかをどう判断するか
- incident 時に何を最初にするか
- どう recover / rollback するか

## Runbook Section

### Startup

含むべきもの:

- intended topology
- required dependency
- first health check
- safe mode での起動方法

### Live Operation

含むべきもの:

- normal がどう見えるか
- どの warning が重要か
- route / destination health をどこで見るか

### Incident Response

含むべきもの:

- slow destination procedure
- malformed traffic procedure
- node restart recovery procedure
- plugin failure procedure

### Recovery

含むべきもの:

- rehydrate procedure
- replay safety procedure
- cache invalidation procedure
- fallback procedure

### Rollback

含むべきもの:

- config revert 方法
- release profile revert 方法
- last-known-good state への戻し方

## Ownership And Signoff

各 release が定義すべきもの:

- technical owner
- operational owner
- rollback owner

## 非交渉の不変条件

- rollback clarity のない release は完了とみなさない
- safe mode story のない operational profile を出さない
- known limitation は暗黙にせず文書化する
- release は developer だけでなく operator に理解可能であること
