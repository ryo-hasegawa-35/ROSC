# Operator Workflow And Recovery Playbook

## 目的

この文書は、normal operation、degraded operation、recovery 時に
operator が broker をどう扱うべきかを定義します。

目標は、高度機能を本番の緊張下でも運用可能にすることです。

## プロダクトが答えるべき Operator の問い

- 今何が起きているか
- 何が不健全か
- 何が drop されているか
- 何が変わったか
- どうすれば安全に復旧できるか

## Normal Workflow

### Startup

operator は起動時に次を選べるべきです。

- normal mode
- safe mode
- recovery mode

### Preflight

preflight が見せるべきもの:

- listening endpoint
- route count
- disabled route
- discovery status
- security status
- stale cache warning

### Live Monitor

live monitor が見せるべきもの:

- overall state: healthy / pressured / degraded / emergency
- top queue growth
- drop reason
- breaker event
- destination health
- plugin health

## Incident Playbook

### Playbook A: Slow Destination

症状:

- egress queue の増加
- destination timeout の繰り返し

action:

1. destination health を確認
2. breaker state を確認
3. 必要なら destination を isolate
4. healthy route が安定していることを確認

### Playbook B: Sensor Flood

症状:

- sensor route に pressure が集中
- control traffic が危険

action:

1. traffic class を確認
2. shedding policy が動いていることを確認
3. 設定されていれば sensor stream を sample / coalesce
4. critical control が守られているか確認

### Playbook C: Malformed Traffic Storm

症状:

- parse error
- quarantine trigger

action:

1. offending source を確認
2. quarantine または rate limit が効いていることを確認
3. healthy traffic が継続していることを確認
4. 必要なら後解析用に sample capture を取る

### Playbook D: Node Restart Recovery

症状:

- downstream node reconnect
- restart 後に state が欠落

action:

1. route cache policy を確認
2. route または namespace 単位で rehydrate を発行
3. state が戻ったことを確認
4. debug が目的でない限り replay は避ける

### Playbook E: Plugin Failure

症状:

- transform timeout
- plugin disconnect

action:

1. plugin status を確認
2. failure が続くなら plugin を disable
3. core routing を継続
4. 検証後にのみ再有効化

## Recovery Control

UI または operator surface が持つべき操作:

- isolate route
- disable destination
- resend latest cached state
- resend snapshot set
- start sandbox replay
- invalidate cache
- enter safe mode
- acknowledge fault

## Safe Mode

safe mode は次を行うべきです。

- experimental plugin を無効化
- optional で risky な transform を無効化
- core compatible routing を維持
- reduced capability で動作中であることを明確に表示

## Replay Safety

replay の default:

- sandbox target
- marked lineage
- operator-confirmed scope

replay は通常の live traffic を装ってはなりません。

## Audit And History

operator が確認できるべきもの:

- config change history
- 誰が recovery action を起こしたか
- どの route が isolate されたか
- safe mode に入った時刻
- 何が replay されたか

## 非交渉の不変条件

- operator action は可視かつ audit 可能であること
- recovery action は live routing と明確に区別されること
- safe mode は最小限有用な compatible system を維持すること
- recovery を tribal knowledge に頼らせず、製品側が guide すること
