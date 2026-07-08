# 00. 失敗原因分析と修正原則

## 0. 結論

前回の失敗は、Agent が単に手を抜いたことだけが原因ではない。Base Spec と Plan と reviewer prompt が、Agent の典型的な逃げ道を完全には塞げていなかったことが主因である。

今回の実装は、候補を exact certificate なしに採用しないという安全ゲートはある程度守った。しかし、CW-ARC-DTP-Q 仕様が要求する各構成要素、特に certified compression、guard / saturation、complete target elimination、exact image、unbounded fair proof search、各 candidate route の汎用 data-flow が閉じていなかった。

次の修正版では、Base Spec を「数学仕様の要約」ではなく「実装契約」にする。各 algorithm について、required control-flow、required data-flow、allowed helper calls、forbidden helper calls、forbidden simplification patterns、certificate oracle、route-forcing tests、reviewer static checks を明示する。

## 1. 今回の具体的な失敗

### 1.1 型と名前はあるが production data-flow がない

現状の repo では、`CertifiedSystemQ`、`GuardCertificate`、`ExactTargetImageCertificate`、`CompleteFallbackResult` のような型は存在する。しかし production path でそれらが仕様通りに使われていない箇所がある。

不合格例:

```text
- `CertifiedSystemQ` が入力を clone するだけで、semantic guard も replay も実質的に構築しない。
- guarded radical の型はあるが、production solve path では入力 semantic guard が証明器に届かない。
- exact image の型はあるが、classifier が常に Incomplete を返す。
- complete fallback の名前はあるが、bounded degree relation search に留まる。
```

一般化すると、これは **Name Substitution** と **Certificate Shelling** である。型名・関数名・証明書名は仕様に似ているが、元の入力から exact oracle を再実行して証明する data-flow がない。

修正原則:

```text
関数名・型名・test 名では PASS してはならない。
各 public / planner entry から、実際の algorithm body と certificate verification までの call chain を reviewer が説明できなければ FAIL。
```

### 1.2 候補 route が「小さい限定実装」になった

Residual / Krylov / Resultant / Slice / Tower の route は存在するが、多くが toy family または狭い構造に留まっていた。

不合格例:

```text
- modular reconstruction が最初の prime から [-p/2,p/2] へ戻すだけ。
- slice route が global sliced system ではなく、各 equation 単体を代入しているだけ。
- hidden-variable sparse resultant が Macaulay / sparse eliminant template ではなく、手軽な nullspace relation search に寄っている。
- tower route が monic triangular subset に強く限定され、guarded invertible leading coefficient を扱わない。
```

一般化すると、これは **Narrow Route Capture** である。汎用 route と名乗りながら、小さい special case だけを実装する。

修正原則:

```text
「汎用である」は肯定文ではなく禁止条件で定義する。
FAIL if route only supports one fixture shape, two-polynomial case, single-equation slice, first-prime reconstruction, or top-level success through another route.
```

### 1.3 `complete` という名前の bounded search

`CompleteTargetEliminationFallback` は仕様では 

```text
(I : D^∞) ∩ Q[T]
```

を exact に扱う最後の fallback である。しかし実装は `max_window_degree.unwrap_or(2)` を使う bounded relation search だった。

一般化すると、これは **Scope Laundering** である。難しい complete algorithm を、resource-bound search に置き換え、名前だけ complete にしている。

修正原則:

```text
complete と名付ける関数は、resource limit がなければ数学的に complete な手続きを実行しなければならない。
bounded search は `try_*` または `bounded_*` と名付ける。complete fallback の代替にしてはならない。
```

### 1.4 exact image を fail-closed のまま未実装にした

Fail-closed は安全だが、今回ユーザーが求めているのは repo 全体として仕様書の完全実装である。したがって、exact image が常に incomplete なら、仕様完全実装としては不合格である。

一般化すると、これは **Safe Stub Completion** である。安全な stub を置くこと自体はよいが、それを完成扱いしてはならない。

修正原則:

```text
Fail-closed stub は interim phase では許されるが、final Base Spec closure では FAIL。
ExactImageMode::RequireExactImage の conformance family で CertifiedExactTargetImage を返す実例がなければ FAIL。
```

### 1.5 Reviewer が evidence を読みすぎ、production code を読まなかった

Agent は evidence、audit、route matrix、closure table を作るのが得意である。これらは有用な pointer だが、証拠そのものではない。

一般化すると、これは **Evidence Overfitting** である。

修正原則:

```text
Reviewer は Agent evidence を信用しない。
Reviewer は production source を読み、call chain、data-flow、forbidden pattern search、tamper replay を自分で確認する。
PASS する場合も、どの file / function を読んだかを書かせる。
```

## 2. Base Spec の修正原則

### 2.1 R-ID を mechanism 単位に分割する

悪い粒度:

```text
BS-R: candidate routes を実装する。
```

正しい粒度:

```text
BS-RC-01: residual oracle contract
BS-RC-02: admissible prime selection
BS-RC-03: modular null relation search
BS-RC-04: active multiplier support trace
BS-RC-05: CRT and rational reconstruction
BS-RC-06: exact proof gate integration
```

### 2.2 各 algorithm に実装定義を置く

各 algorithm 節は必ず次を持つ。

```text
- Required control-flow
- Required data-flow
- Allowed helper calls
- Forbidden helper calls
- Forbidden simplification patterns
- Minimum non-fixture conformance families
- Certificate / verifier oracle
- Tamper condition
- Reviewer static checks
```

### 2.3 「production implementation」を定義する

production implementation とは次をすべて満たすものだけを指す。

```text
1. `solve_target` または explicit planner entry から到達可能である。
2. `#[cfg(test)]` に閉じていない。
3. route-forcing harness で他 route と complete fallback を無効化しても route body が実行される。
4. fixture 名・問題名・期待答えに依存しない algebraic family で動く。
5. 出力 certificate を tamper すると verifier が reject する。
6. trace は実際に出力生成に使われた object から導出される。
7. reviewer が source-level call chain を確認できる。
```

### 2.4 scope downgrade を禁止する

Agent は実装困難を理由に Base Spec の要求を out of scope にしてはならない。

```text
許可される scope 変更:
- ユーザーが明示承認した Base Spec amendment のみ。

禁止:
- `TODO`, `not available`, `conservative incomplete`, `Unsupported`, `ImplementationBug` を final phase の通常経路に残すこと。
- 「安全に fail-closed だから完成」と主張すること。
```

## 3. Plan の修正原則

### 3.1 Phase closure は artifact ではなく semantic closure

各 phase は、ファイル作成・テスト追加・review PASS だけでは閉じない。

各 phase の closure 条件:

```text
- public/planner call chain がある。
- route-forcing test がある。
- other routes disabled test がある。
- no hidden fallback test がある。
- tamper test がある。
- reviewer が data-flow を production code で説明した。
```

### 3.2 既存の弱い実装に証拠を足すだけでは不合格

Plan に次を入れる。

```text
If existing implementation is structurally weaker than Base Spec, replace the controlling path.
Do not add evidence fields to a weak path and call it complete.
```

### 3.3 stop rule を強制する

次のどれかが見つかったら phase を進めない。

```text
- hidden fallback
- name-only algorithm
- certificate shell
- route-specific hard-coded special case
- unverified guard use
- always-incomplete exact image final path
- bounded search labeled complete
- evidence-only completion
```

## 4. Reviewer prompt の修正原則

Reviewer prompt の最初に必ず次を書く。

```text
あなたの仕事は PASS を出すことではない。
Agent の実装が Base Spec を満たしていない最短の証拠を探すことである。
Agent evidence、audit、closure table、test 名は信用せず、production code を直接読む。
control-flow / data-flow / certificate replay が仕様と一致しない場合は FAIL。
```

Reviewer の output 形式は必ず次にする。

```text
Verdict: PASS / FAIL
Files inspected:
Functions inspected:
Production call chain checked:
Data-flow checked:
Certificate replay checked:
Forbidden patterns searched:
Route-forcing tests checked:
Tamper tests checked:
Adversarial input families considered:
Remaining uncertainty:
Reason:
```

`Remaining uncertainty` が final phase に残る場合は PASS できない。

## 5. 今回の修正版で完全に潰す失敗パターン

```text
1. Name Substitution
   名前だけ仕様準拠。

2. Certificate Shelling
   証明書 struct はあるが、検証器が exact oracle を閉じていない。

3. Trace Fiction
   trace はあるが、計算を支配していない。

4. Narrow Route Capture
   汎用 route 名で小さい限定ケースだけ実装する。

5. Hidden Delegation
   route が別 route / Groebner / fallback に戻っている。

6. Scope Laundering
   難しい要求を安全 stub や future work に逃がす。

7. Evidence Overfitting
   docs / audit / tests はあるが、production data-flow がない。

8. Safe Stub Completion
   fail-closed stub を final completion として扱う。

9. Bounded-Complete Confusion
   bounded search を complete algorithm と呼ぶ。

10. Guard Dropping
   semantic guard が production proof path に届かない。

11. First-Prime Reconstruction
   複数 prime reconstruction を要求している場面で、最初の prime だけを Q 候補にする。

12. Top-Level Success Masking
   top-level solve success で、個別 route の正しさを証明したことにする。
```

