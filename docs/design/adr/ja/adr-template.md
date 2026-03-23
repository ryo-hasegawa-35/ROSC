# ADR Template

新しい ADR を作るときは、このファイルをコピーして
`adr-XXXX-short-title.md` にリネームしてください。

# ADR-XXXX Title

- Status: proposed | accepted | superseded
- Date: YYYY-MM-DD
- Related issues:
- Related docs:
- Supersedes:

## Context

この決定が必要になった問題、曖昧さ、判断圧力を書きます。

## Decision

採用する決定内容を明確かつ具体的に書きます。

## Consequences

良い影響と悪い影響の両方を書きます。

## Compatibility Impact

次のどれに効くかを明記します。

- strict behavior
- legacy-tolerant behavior
- extended behavior
- compatibility semantics には影響しない

## Recovery, Rollback, Or Operational Impact

operator-visible な影響、rollback の必要性、recovery への影響、または影響が
ない理由を書きます。

## Rejected Alternatives

検討した代替案と、採用しなかった理由を書きます。

## Follow-Up

この ADR から続いて必要になる docs、issue、実装作業を書きます。
