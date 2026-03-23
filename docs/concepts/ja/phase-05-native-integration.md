# Phase 05: ネイティブ統合

## 目標

共有メモリ IPC とホストネイティブ統合により、ローカル高性能ワークフローでは
UDP をバイパスできるようにする。

## 成果物

- Rust core を包む C ABI wrapper
- local machine communication 向け shared memory transport
- IPC 向け lock-free または low-contention ring buffer design
- UE5 native plugin
- TouchDesigner native bridge strategy
- native transport が使えない場合の standard OSC fallback path
- local latency / jitter 計測用 validation tooling

## 設計上の制約

- native integration は唯一の経路にしてはならない
- shared memory path でも network path と同じ論理 routing semantics を保つ
- native integration が失敗しても standard OSC または代替 local IPC に
  graceful に落ちるようにする

## 推奨ロールアウト

1. shared memory proof of concept
2. C ABI stabilization
3. UE5 plugin integration
4. TouchDesigner integration
5. performance / reliability soak test

## 非目標

- UDP code path の削除
- すべてのユーザーが native plugin を入れられる前提
- crash recovery plan のない危険な cross-process memory trick

## 完了条件

- 同じ project が standard OSC mode と native IPC mode の両方で動く
- UE5 が UDP より明確に低い local jitter でメッセージ交換できる
- native plugin が restart、reconnection、fallback を含む運用に耐える

## 概算工数

220-400 時間

## 価値

このフェーズで、プロジェクトは通常の OSC ルーターと本当に差別化されます。
ただし同時に、platform-specific な保守が現実の課題になります。
