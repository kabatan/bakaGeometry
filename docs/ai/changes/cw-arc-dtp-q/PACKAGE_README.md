# CW-ARC-DTP-Q Guardian Implementation Package

このパッケージには、空の Rust repo から `geosolver-core` の CW-ARC-DTP-Q candidate-cover solver を実装させるための Guardian 用文書が入っています。

## Files

- `CW_ARC_DTP_Q_GUARDIAN_BASE_SPEC.md`
  - 最終的に repo が満たすべき仕様です。
  - 添付 CW-ARC-DTP-Q v2 の数学的仕様を中核に、file 構成、型、関数、禁止実装、検証条件まで固定しています。

- `CW_ARC_DTP_Q_GUARDIAN_PLAN.md`
  - Base Spec を実装するための phase-by-phase plan です。
  - 各 phase に R-ID、MECH、実装対象 file、受け入れ条件、停止条件、reviewer prompt を割り当てています。

- `CW_ARC_DTP_Q_GUARDIAN_REVIEWER_PROMPTS.md`
  - Guardian reviewer に渡す prompt 集です。
  - Agent evidence を信用せず、production code の control-flow / data-flow / certificate replay を直接読むようにしています。

## Recommended Guardian flow

1. User approves the Base Spec and Plan.
2. Agent copies the Base Spec and Plan into `docs/ai/changes/cw-arc-dtp-q/`.
3. Agent executes phases P0..P15 in order.
4. At every phase, run the matching reviewer prompt.
5. Do not close a phase from tests alone.
6. Final closure must not claim general exact image unless real-fiber classification is actually implemented and reviewed.
