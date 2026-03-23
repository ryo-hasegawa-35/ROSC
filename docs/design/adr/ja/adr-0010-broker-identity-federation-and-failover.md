# ADR-0010: Broker Identity, Federation, And Failover

- Status: accepted
- Date: 2026-03-23

## Context

federated / standby deployment では、
ownership と failover rule が explicit でないと split-brain と authority 混線が起きます。

## Decision

- explicit な broker identity を持つ
- replication scope と ownership boundary を定義する
- より複雑な topology の前に active/standby semantics を採用する
- failover authority と split-brain prevention を explicit にする

## Consequences

- fault condition でも operations を理解しやすい
- replication logic が declared scope の下に置かれる
- multi-broker deployment には厳密な coordination rule が必要になる

## Rejected Alternatives

- implicit multi-master behavior
- discovery だけに依存した failover authority

## Affected Documents

- [Federation And High-Availability Model](../../ja/federation-and-high-availability-model.md)
- [Profile-Specific Operator Guides](../../ja/profile-specific-operator-guides.md)
- [Release Checklist And Operational Runbook](../../ja/release-checklist-and-operational-runbook.md)
