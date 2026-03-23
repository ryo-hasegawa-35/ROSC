# Phase 04: 拡張性、スキーマ、コード生成

## 目標

高度機能をプラガブルにして、コアを肥大化させずにシステムを成長させる。

## 成果物

- feature flag 付き product preset
- runtime Wasm filter engine
- 承認済み filter module の hot reload
- out-of-process extension 向け stable external plugin protocol
- 以下を含む schema definition format:
  - addresses
  - argument types
  - units
  - constraints
  - documentation
- validation / lint tooling
- code generation target:
  - Rust bindings
  - C ABI descriptors
  - UE5 向け C++ wrapper
  - TouchDesigner 向け Python helper

## プラグイン方針

- packet transform や計算拡張は Wasm を使う
- protocol bridge や重量級 connector は外部プロセス型 plugin にする
- メインの拡張機構としては、不安定な native Rust plugin ABI に依存しない

## スキーマ方針

- まずは documentation と validation から始める
- 実プロジェクトで価値が確認できてから code generation を加える
- ad-hoc OSC を殺さないため、schema は optional に保つ

## 非目標

- legacy user への schema registration 強制
- すべての message の事前宣言義務化
- 現場の ad-hoc workflow を全面的に置き換えること

## 完了条件

- ユーザーが broker 全体を再コンパイルせず custom packet transform を追加できる
- schema により一般的な型違いを runtime 前に検出できる
- 少なくとも 1 つの実ユースケースで、生成コードが UE5 または
  TouchDesigner 連携の定型作業を減らせる

## 概算工数

160-300 時間

## 価値

このフェーズで、システムは単なる「自前ルーター」ではなく
拡張可能な基盤へ変わります。
