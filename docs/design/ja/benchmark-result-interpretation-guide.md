# Benchmark Result Interpretation Guide

## 目的

この文書は、benchmark の結果をどう読むべきかを定義します。

workload definition は「何を走らせるか」を決めます。この文書は、その結果が
本当に良いのか、危険なのか、誤解を招くのか、release-blocking なのかを
判断するための基準です。

関連文書:

- [Benchmark Workload Definition](./benchmark-workload-definition.md)
- [Metrics And Telemetry Schema](./metrics-and-telemetry-schema.md)
- [Compatibility Matrix](./compatibility-matrix.md)
- [Release Checklist And Operational Runbook](./release-checklist-and-operational-runbook.md)

## Benchmark は Speed だけでなく Trust を測る

速い結果がそのまま良い結果とは限りません。

解釈では常に次を問うべきです。

- compatibility は正しく保たれたか
- route isolation は維持されたか
- overload behavior は explicit なままだったか
- observability overhead は許容範囲か
- recovery behavior は正しかったか

## どの結果にも必要な Context

すべての benchmark report は次を含むべきです。

- workload definition version
- software revision または document revision
- operating system
- CPU class
- memory class
- active feature toggle
- release profile
- route count と destination count
- warm run か cold run か

これが欠けている結果は不完全です。

## 最初に問うべき判断軸

### Question 1: Correctness は保たれたか

次のどれかがあれば即 reject か investigate 対象です。

- 宣言された drop policy 以外で packet が失われた
- compatibility mode の挙動が想定外に変わった
- malformed traffic が crash、stall、hidden corruption を起こした

### Question 2: Predictability は保たれたか

見るべきもの:

- p95 / p99 latency の伸び
- jitter の広がり
- queue depth growth pattern
- fault 時の breaker behavior

速くても chaotic な system より、予測可能に degrade する system の方がよいです。

### Question 3: Isolation は保たれたか

routing で最も重要な問いの一つは次です。

- unhealthy / slow path が局所化されたか

slow destination が healthy critical path へ影響したなら、headline throughput が
上がっていても major regression とみなすべきです。

### Question 4: Recovery は有用なままか

確認すべきもの:

- rehydrate latency
- cache correctness
- restart recovery time
- replay safety boundary

速くても incorrect な recovery は失敗です。

## 主要 Signal の読み方

### Throughput

Throughput は二次指標として扱います。

packet per second の改善が意味を持つのは、次が同時に成り立つ場合です。

- compatibility が正しい
- tail latency が許容範囲
- drop reason が policy 内

### Tail Latency

live show では median より p95 / p99 の方が重要です。

解釈ルール:

- median は通常時の姿
- p95 / p99 は土管を信用できるかどうか

### Jitter

raw speed より timing consistency が重要な系では jitter が効きます。

average latency がよくても、mixed load で jitter が広がるなら warning とみなすべきです。

### Queue Depth

Queue depth は drop より先に未来の trouble を教えてくれます。

解釈ルール:

- 浅く安定した queue は headroom を示す
- 継続的に伸びる queue は将来の instability を示す

### Drop Count By Reason

drop はすべて同じ意味ではありません。

区別すべきもの:

- disposable stream に対する intentional sampling
- critical route 上の overload drop
- malformed packet rejection
- security rejection

critical route の drop は、profile policy で明示的に許可されていない限り
release-blocking に近い扱いにすべきです。

### Breaker Event

breaker open は自動的に失敗を意味しません。

解釈ルール:

- optional analytics path の breaker open は isolation が効いている証拠になりうる
- primary control path の breaker open は serious incident indicator

### CPU And Memory

resource use が高いこと自体は即失格ではありません。ただし次と結びつくと重要です。

- tail latency が悪化している
- diagnostics 自体が bottleneck になっている
- smaller machine も profile target に含まれる

## Run 同士を安全に比較する方法

次が概ね揃っているときだけ比較すべきです。

- workload version
- platform class
- feature toggle
- profile
- route count と destination count
- warm-up behavior

推奨する比較手順:

1. 同じ workload を複数回走らせる
2. median と tail behavior を run 間で比べる
3. improvement を主張する前に variance を調べる

## Result Class

### Pass With Headroom

次を満たすとき:

- correctness が保たれている
- tail latency が bounded
- isolation が保たれている
- resource growth が許容範囲

### Pass With Caution

次を満たすとき:

- correctness は保たれている
- ただし resource cost や telemetry cost が増えた
- degradation は explicit かつ bounded

### Investigate Before Claiming Improvement

次のとき:

- headline throughput は上がった
- しかし tail latency、jitter、queue growth が悪化した
- または run 間 variance が大きい

### Fail

次のどれかがあるとき:

- compatibility が壊れた
- isolation が壊れた
- crash または hidden corruption が起きた
- recovery が incorrect になった

## よくある誤読

- throughput 改善だけ見て tail latency を無視する
- workload version が違う run を同列比較する
- metrics や capture を有効にしたことで measurement target 自体が変わるのを忘れる
- sampled drop と critical drop を同じ意味で扱う
- たまたま良かった 1 run で unstable variance を隠す

## Release Evidence Rule

release claim には次を添えるべきです。

- どの benchmark workload を使ったか
- どの profile と toggle を使ったか
- interpretation class は何か
- 既知の caveat は何か

benchmark は marketing 用数字ではなく、honest な release note を支えるべきです。

## Non-Negotiable Rule

- 十分な context がない benchmark result を提示してはいけない
- Compatibility regression は speed improvement より重い
- Isolation regression は headline throughput gain より重い
- Tail behavior と recovery behavior は first-class な release criterion である
