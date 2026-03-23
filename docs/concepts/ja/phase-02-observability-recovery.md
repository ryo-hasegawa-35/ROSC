# Phase 02: 可観測性と復旧

## 目標

ライブ環境でも broker が見え、調査でき、復旧できる状態にする。

## 成果物

- embedded dashboard backend / frontend
- real-time traffic graph
- route topology visualization
- endpoint ごとの health / throughput view
- stateful last-value cache
- late joiner sync
- recent traffic を保持する in-memory ring buffer
- capture / replay tooling
- time-travel debug workflow:
  - packet history の確認
  - address / route / source での絞り込み
  - 安全な replay
- config snapshot history
- correlation ID 付き structured logs

## プロダクト上の決定事項

- ring buffer の retention strategy
- デフォルトで cache して良い値の範囲
- replay を live output からどう隔離するか
- dashboard を read-only にするか operational にするか

## 運用上の安全策

- replay は sandbox または dry-run をデフォルトにする
- cache sync は route または namespace 単位の opt-in にする
- 診断機能が fast path を大きく劣化させないようにする

## 非目標

- 汎用 plugin marketplace
- secure multi-tenant mode
- shared memory IPC

## 完了条件

- operator が packet sniffer なしで bottleneck や drop を特定できる
- 再起動した node が cache policy によって状態復旧できる
- 収集した問題を制御された形で replay してデバッグできる

## 概算工数

120-180 時間

## 価値

このフェーズで broker はブラックボックスではなく運用ツールになります。
現場で信頼されるかどうかは、ここで大きく変わります。
