# Profile-Specific Operator Guides

## 目的

この文書は、deployment profile ごとに operator が何を優先すべきかを
整理するための guide です。

同じ broker core でも、運用のしかた次第で安全にも危険にもなります。ここでは
major profile ごとに、incident が起きる前から何を見てどう動くべきかを定義します。

関連文書:

- [Deployment Topology And Release Profile Guide](./deployment-topology-and-release-profile-guide.md)
- [Dashboard Interaction Spec And Screen Inventory](./dashboard-interaction-spec-and-screen-inventory.md)
- [Operator Workflow And Recovery Playbook](./operator-workflow-and-recovery-playbook.md)
- [Release Checklist And Operational Runbook](./release-checklist-and-operational-runbook.md)

## すべての Operator が答えられるべき質問

- この profile で本当に mission-critical な traffic class は何か
- pressure 時に最初に捨ててよい destination はどれか
- safe mode へどう落とすか
- recovery 時に何の evidence を残すべきか
- 何が automatic で、何が operator confirmation を要するか

## Guide 1: `core-osc` On A Localhost Sidecar

使う場面:

- creative tool と broker を 1 台で動かす
- 導入 friction を最小にしたい
- raw OSC compatibility を最優先したい

operator の主眼:

- main control path に余計な latency を載せないこと

healthy signal:

- queue depth が低く安定している
- parse failure がほぼゼロ
- 想定外の breaker event がない
- localhost latency が負荷下でも安定している

incident 時の最初の action:

1. ingress parse failure と destination breaker state を確認する
2. 不要な tap や dashboard subscriber を止める
3. observability overhead が疑わしければ minimal safe profile に落とす

safe mode:

- core routing のみ
- heavy capture なし
- minimal metrics

最初に最適化しないもの:

- discovery
- schema tooling
- dashboard の見た目

## Guide 2: `ops-console` On A Single Workstation Hub

使う場面:

- 1台の machine が複数の local / network peer を中継する
- rehearsal や本番で browser console を常用する

operator の主眼:

- diagnostics を有用に保ちつつ、diagnostics 自体を outage 要因にしないこと

healthy signal:

- telemetry route が悪化しても control route は健全のまま
- dashboard tap latency が operator tolerance 内
- route diff event に説明可能性と監査性がある

incident 時の最初の action:

1. unhealthy state が ingress 側か destination 側かを切り分ける
2. 必要なら high-rate route の diagnostics level を落とす
3. critical control route を触る前に slow consumer を隔離する

safe mode:

- 非必須 screen を止める
- route health と breaker state は見えるままにする
- replay は manual のままにする

## Guide 3: `ue5-workstation` For Local Or Dual-Machine Show Work

使う場面:

- UE5 が主要 runtime
- camera、scene、cue 系 traffic を tight に保ちたい

operator の主眼:

- engine restart や hot reload 時に timing と state continuity を守ること

healthy signal:

- stateful control cache が新鮮
- rehydrate が速く bounded
- IPC または localhost route に説明不能な spike がない

incident 時の最初の action:

1. 問題が engine 側か broker 側か IPC 境界かを確認する
2. rehydrate 前に cache freshness を確認する
3. blind replay ではなく controlled rehydrate を優先する

safe mode:

- IPC が疑わしければ UDP path へ戻す
- experimental transform を止める
- 既に検証済みの state recovery 機能だけ残す

## Guide 4: `touchdesigner-kiosk` For High-Rate Sensor Work

使う場面:

- TouchDesigner などが dense な sensor stream を受ける
- bounded loss は許容できるが、不安定さは許容できない

operator の主眼:

- sensor storm から control traffic を守ること

healthy signal:

- sensor route は policy の範囲で sample / drop してよい
- sensor route が悪化しても critical control route は安定
- capture は bounded のまま

incident 時の最初の action:

1. どの traffic class が影響を受けているか確認する
2. optional monitor や browser subscription を減らす
3. route を detailed から minimal metrics へ落とすべきか確認する

safe mode:

- critical control と stateful route を残す
- sensor observability を先に degrade する
- malformed source は積極的に quarantine する

## Guide 5: `secure-installation` On A Segmented Network

使う場面:

- shared network や semi-hostile network を使う
- source verification と auditability が重要

operator の主眼:

- legacy peer を壊さずに trust boundary を明示し続けること

healthy signal:

- rejected-source count に説明がつく
- verified bridge と legacy bridge が明確に分離されている
- discovery state が承認済み topology と一致している

incident 時の最初の action:

1. 認証 failure か routing failure かを切り分ける
2. source が unknown、stale、revoked のどれかを確認する
3. 大きく rollback する前に audit evidence を保全する

safe mode:

- secure ingress enforcement は維持する
- optional discovery を止める
- 既に承認済みの compatibility-only bridge は残す

## Guide 6: Active / Standby Pair

使う場面:

- 最小構成より continuity を優先する
- config と一部 recovery state を複製する

operator の主眼:

- failover 時に ownership が曖昧にならないこと

healthy signal:

- primary / standby identity が曖昧でない
- replication lag が宣言済み tolerance 内
- failover state transition が明示 event として見える

incident 時の最初の action:

1. primary が本当に unhealthy なのか、ただ partition されているだけかを判定する
2. promote 前に standby freshness を確認する
3. failover reason code と operator note を残す

safe mode:

- risky config change を凍結する
- identity が曖昧なら automatic promotion より manual failover を優先する
- namespace ごとに明示検証されていない限り replay は無効のまま

## Guide 7: Federated Brokers

使う場面:

- 複数 broker が意図的に traffic や state を交換する
- site ごとにある程度自律性を保ちたい

operator の主眼:

- ある site の disorder が全体へ伝播しないこと

healthy signal:

- replication scope が selective で universal ではない
- remote lag と breaker state が peer ごとに見える
- remote degradation があっても local safety policy が崩れない

incident 時の最初の action:

1. 問題が local、remote、federation link のどこかを切り分ける
2. local degradation を広げる前に replication scope を狭める
3. cross-site confidence が低いなら local critical traffic を authoritative に保つ

safe mode:

- critical namespace は local-only routing に戻す
- 非 critical federation link を停止する
- forensic evidence は bounded にし、broker identity を必ず付与する

## Profile をまたいで守るべき Invariant

- Critical control は optional telemetry と同じ failure budget を共有しない
- Safe mode は単に生存するだけでなく、operator が理解できる状態を保つ
- Stateful namespace では replay より rehydrate を優先する
- Incident evidence には route、destination、broker identity、compatibility mode を含める

## Hand-Off Rule

もし profile guide が normative な design document と矛盾したら、この
guide 側を直すべきです。profile は tuning のためのものであって、core safety
semantics を作り替えるためのものではありません。
