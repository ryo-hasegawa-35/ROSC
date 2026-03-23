# Benchmark Fixture Inventory And Reproducibility Plan

## 目的

この文書は、performance claim を出す前に存在すべき benchmark input class と
reproducibility rule を凍結するためのものです。

## Fixture Root

repository の fixture root は次です。

- `fixtures/benchmarks/`

主な inventory file:

- `fixtures/benchmarks/workload-catalog.json`
- `fixtures/benchmarks/context-template.json`

## Workload Class

最初の benchmark catalog には次を含めるべきです。

- control-plane message burst
- dense sensor stream
- mixed installation traffic
- malformed pressure traffic
- recovery と late-joiner traffic
- adapter bridge traffic

## Reproducibility Rule

将来の benchmark run は毎回、少なくとも次を記録するべきです。

- git revision
- build profile
- operating system と hardware note
- workload ID
- packet-size assumption
- destination count
- timing source と duration

## Interpretation Rule

benchmark は、一度に 1 つの named question に答えるべきです。例:

- throughput ceiling
- p99 queue latency
- overload containment behavior
- recovery overhead

これらを 1 つの headline number に潰してはいけません。

## 初期 Inventory

最初の workload inventory は次に置きます。

- `fixtures/benchmarks/workload-catalog.json`

必要な metadata template は次に置きます。

- `fixtures/benchmarks/context-template.json`
