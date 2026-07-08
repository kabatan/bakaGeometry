# CW-ARC-DTP-Q Full Guardian Package v3

この package は、前回の実装失敗を前提に、Guardian Agent に渡すための修正版 Base Spec / Plan / Reviewer Prompt 一式である。

## ファイル

1. `00_FAILURE_ANALYSIS_AND_FIX_PRINCIPLES.md`  
   失敗原因の分析と、それを一般化した再発防止原則。

2. `01_BASE_SPEC_CW_ARC_DTP_Q_FULL_V3.md`  
   repo が最終的に満たすべき状態を、file / module / function / algorithm data-flow まで固定した Base Spec。

3. `02_PLAN_CW_ARC_DTP_Q_FULL_V3.md`  
   空 repo または既存 repo から、Base Spec 通りに作るための作業計画。

4. `03_REVIEWER_PROMPTS_CW_ARC_DTP_Q_FULL_V3.md`  
   Guardian reviewer に渡す phase 別 prompt。Agent evidence を信用せず、production code を直接読む前提。

5. `04_ROUTE_CHECKLISTS_AND_TEST_MATRIX.md`  
   route-forcing / no-fallback / tamper / non-simplification の検査表。

## 使い方

Guardian の Base Spec には `01_BASE_SPEC_CW_ARC_DTP_Q_FULL_V3.md` を使う。  
Guardian の Plan には `02_PLAN_CW_ARC_DTP_Q_FULL_V3.md` を使う。  
各 phase の reviewer prompt は `03_REVIEWER_PROMPTS_CW_ARC_DTP_Q_FULL_V3.md` から該当節をそのまま渡す。  
Phase closure では `04_ROUTE_CHECKLISTS_AND_TEST_MATRIX.md` の表を必ず埋めるが、表だけで PASS してはならない。Reviewer は production code の control-flow / data-flow / certificate replay を直接確認する。

