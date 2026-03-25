# Security Overlay Model

## 目的

この文書は、raw OSC compatibility を壊さずに security を追加する方法を
定義します。

このプロジェクトにおける security は overlay と policy layer であり、
plain OSC の置き換えではありません。

## コア原則

- raw OSC はそのまま使える
- secure ingress は additive
- broker が security を終端してから legacy peer へ forwarding する

## Security Objective

- trusted sender を識別する
- traffic を project / environment 単位で scope する
- route access を authorize する
- secure mode で spoofing を防ぐ
- 必要な場合は compatibility route を残す

## Security Domain

### Ingress Identity

ありうる identity:

- anonymous legacy sender
- verified operator client
- verified broker peer
- verified adapter または service

### Scope

ありうる scope:

- project
- venue
- workstation
- namespace

### Authorization

ありうる permission:

- send
- receive
- observe
- transform
- administer

## Secure Envelope Model

secure route は outer envelope または authenticated transport context を
使ってよく、その中に最低限次を含めます。

- sender identity
- scope
- issued-at
- expiry
- signature または token proof

raw OSC payload はその secure envelope の内側にあり、legacy compatibility の
ために payload 自体は改変しません。

## Compatibility Bridge

compatibility bridge の流れ:

1. secure ingress が broker に届く
2. broker が identity と scope を verify
3. broker が route access を authorize
4. 許可された場合のみ downstream に plain な互換 OSC を forwarding

これにより legacy endpoint を変更せずに済みます。

## Route Security Policy

各 route は最低限次を宣言できるべきです。

- `scope`
- `require_verified_source`
- `allowed_identities`
- `allow_legacy_bridge`
- `audit_level`

## Failure Behavior

secure route failure は fail closed であるべきです。

例:

- invalid signature
- expired token
- mismatched scope
- unauthorized sender

一方、compatibility route は通常の route fault policy に従って動き続けても
よいです。

## Replay Protection

secure route は少なくとも次を支えるべきです。

- issued-at validation
- expiry validation
- 必要に応じた nonce または sequence tracking
- operator-driven diagnostics 用の replay-session marking

## Audit Model

broker が最低限記録すべきもの:

- 誰が traffic を送ったか
- どの scope を主張したか
- verification が通ったか
- authorization が通ったか
- どの route decision になったか

## Recovery との関係

recovery は security scope を尊重する必要があります。

ルール:

- secure cached state は scope metadata を保持する
- scope をまたぐ resend は explicit authorization を要求する
- standby broker へ state replicate する場合は security lineage を保つ

## Operator Control

有用な control:

- active identity の確認
- denied event の確認
- secure route の一時停止
- safe mode への移行
- config change の audit

## Jitter Budget Requirement

security feature は cost を増やしてよいですが、jitter への影響を隠しては
いけません。

含意:

- secure mode は plain compatibility mode と分けて benchmark する
- median だけでなく p95 / p99 latency と jitter spread を必ず出す
- frame-accurate な AV sync route は、bulk telemetry や operator traffic より
  厳しい security profile を要求しうる
- verification や envelope handling が route の jitter budget に収まらないなら、
  security を暗黙に有効とみなすのではなく deployment warning として明示する

## 非交渉の不変条件

- legacy peer 向け raw OSC payload に mandatory security field を混ぜない
- secure verification failure を trusted traffic に黙って downgrade しない
- security scope は diagnostics と audit に残る
- secure mode は additive であり、compatibility deployment を壊さない
