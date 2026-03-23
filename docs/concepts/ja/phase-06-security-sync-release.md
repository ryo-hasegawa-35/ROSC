# Phase 06: セキュリティ、同期、配布ハードニング

## 目標

オプションのセキュリティ拡張、高度な同期機能、クロスプラットフォームの
本番配布を完成させる。

## 成果物

- zero-trust namespace gateway
- project ID と scoped access policy
- secure route 向け token または signed-envelope verification
- rate limiting と abuse control
- secure / insecure deployment profile
- Ableton Link integration
- timestamp propagation strategy
- sync diagnostics の可視化
- 以下に向けた installer または distributable package:
  - Windows
  - macOS
  - Linux
- 必要に応じた service mode / auto-start support
- soak test と long-run reliability report

## セキュリティ方針

セキュリティは追加的であるべきです。

- legacy OSC は使い続けられる
- secure overlay は broker 境界で終端する
- 下流の legacy tool は plain な互換 OSC を受け取れる

## 同期方針

- timing metadata の伝播と timing execution の保証は別物
- 製品は timing quality と clock assumption を明確に出す
- sync feature は静かに失敗させず、見える形で失敗させる

## クロスプラットフォーム配布作業

- signing と notarization
- installer UX
- bundling する runtime dependency
- service integration
- log file path と config directory
- firewall guidance

## 完了条件

- 互換モードと secured mode の両方で動作できる
- 3 OS で package がきれいに install できる
- 長時間の soak test で realistic workload に対して安定している

## 概算工数

180-320 時間

## 価値

このフェーズにより、OSC が本来持っていた互換性を保ちながら、
荒れた実環境にも投入できるプラットフォームになります。
