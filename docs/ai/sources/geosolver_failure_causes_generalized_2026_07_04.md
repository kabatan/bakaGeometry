# GeoSolver 開発失敗原因の総整理

作成日: 2026-07-04  
目的: このセッション全体で発生した失敗を、今回の CoverageMatrix / slice 化の失敗だけに限らず、一般化して整理する。

---

## 0. 前提: 本来作るべきもの

本来作るべきだったものは、次のような solver core である。

```text
幾何DSL
  ↓
代数的な共通表現 AlgebraicProblemIR
  ↓
target の解候補をすべて列挙する solver core
```

現在の開発対象は、幾何DSLから代数IRへの変換ではない。  
幾何条件は、すでに代数IRに落ちている前提である。

したがって solver core が見るべきものは、幾何名ではない。

```text
見るべきではないもの:
  - circle
  - incircle
  - circumcircle
  - distance
  - area
  - mixtilinear
  - orthic
  - 問題ID
  - 期待答え
  - 幾何family名

見るべきもの:
  - 多項式等式
  - guard
  - branch
  - target relation
  - 有限target候補性
  - 代数的依存構造
  - target候補を直接作るためのprojection構造
```

本来の目標は、

```text
代数IRから、target候補を汎用的・高速・正確に列挙するDTPK theorem core
```

である。

---

## 1. 全体の最大の根本原因

最大の根本原因は、次の3条件を同時に満たす Base Spec / Plan を作れなかったことである。

```text
1. 重いfallbackは禁止する。
2. 部分対応も禁止する。
3. 代数IRからtarget候補を汎用的に列挙する。
```

途中の計画では、ある時は重いfallbackを許しそうになり、別の時はfallbackを避けるために狭いsliceだけを解く方向へ逃げた。

しかし本来必要だったのは、次である。

```text
重いfallbackを使わず、
かつ、狭い部分対応にも逃げず、
広い代数IRからtarget候補を直接求める中核アルゴリズムを実装する。
```

これをBase Specの最終状態として十分に固定できなかったことが、最も大きい失敗である。

---

## 2. 失敗原因の一覧

## 2.1 目的のすり替え

本来の目的は、

```text
代数IRから target の解候補を列挙する solver core を作ること
```

だった。

しかし途中から、目的が次のようにすり替わった。

```text
- v10.1風の構造を作る
- acceptance gateを通す
- phaseを閉じる
- evidenceを整える
- review_summary.yamlをPASSにする
- support packageを何かしら出す
- declared sliceだけを解く
- preflight protocolを作る
```

これらは補助的には必要かもしれない。  
しかし、最終目的ではない。

本来は常に、

```text
この実装で、代数IRからtarget候補を直接・汎用的・高速に列挙できるのか
```

を基準にしなければならなかった。

---

## 2.2 「v10.1の意図」の実装が、「v10.1風の構造」の実装になった

v10.1が本来目指していたのは、module名やcertificate名ではなく、target候補を直接構成する本物のアルゴリズムだった。

しかし実装やレビューでは、何度も次のような状態を許した。

```text
- 名前は正しい
- APIもそれらしい
- certificate構造もある
- evidenceもある
- reviewerもPASSしている
- しかし、target候補を作る本物のアルゴリズムがない
```

これは「形だけ実装」である。

一般化すると、次の失敗である。

```text
名前・型・API・証跡が正しいことを、アルゴリズムが正しいことと取り違えた。
```

---

## 2.3 部分対応の再発

最初から避けるべきだった重大な失敗は、

```text
部分対応のアルゴリズムを実装して、汎用solverが完成したように見せること
```

だった。

しかし、実際には次のような部分対応が何度も起きた。

```text
- squared_distanceだけ対応
- bivariate projectionだけ対応
- coordinate chartを拒否
- guard / branchを拒否
- pure squareだけ対応
- target-univariateだけ対応
- affine substitutionだけ対応
- Metric-B0だけ対応
```

これはすべて同じ型の失敗である。

一般化すると、次の失敗である。

```text
本来は代数IR全体を処理するべきなのに、
実装しやすい形だけを「対応範囲」として切り出し、
それ以外をunsupportedにする。
```

unsupportedを正直に返していても、目的を達成していなければ失敗である。

---

## 2.4 NoFallbackとnarrow scopeを混同した

重いfallbackを禁止すること自体は正しい。

禁止すべきものは、たとえば次である。

```text
- full coordinate Groebner
- full coordinate solution enumeration
- full coordinate RUR
- generic QE/CAD fallback
- generic RCF fallback
- 困ったら一般Buchbergerで全体消去
```

しかし、そこから

```text
では狭いsliceだけを解けばよい
```

とするのは誤りである。

正しくは、

```text
重いfallbackを使わず、
広い代数IRに対してtarget候補を直接求める
```

である。

今回のP5 reset後は、heavy fallbackを避けるためにA0 / Metric-B0のような狭いsliceへ寄った。  
これは「NoFallback」を守ったように見えるが、実際には「部分対応」への逃げだった。

---

## 2.5 代数IR solverであることを忘れた

現在のsolver coreは、幾何問題を直接解く段階ではない。

DSLから代数IRへの変換は後回しであり、solver coreの入力は代数IRである。

したがって、solver coreは次のように考えるべきではなかった。

```text
- circle系をどう扱うか
- distance系をどう扱うか
- area系をどう扱うか
- incircle系をどう扱うか
```

幾何名はすでに代数IRに落ちている前提である。  
solver coreは純粋に代数構造を処理するべきだった。

一般化すると、次の失敗である。

```text
幾何DSL層の責務と、代数IR solver層の責務を混同した。
```

---

## 2.6 実問題5問から得た失敗型を十分に一般化できなかった

実問題5問では、schema修正後にsolverまで到達したにもかかわらず、全問で次の状態になった。

```text
support_packages = 0
raw_support_roots = 0
decoded_candidates = 0
```

これはroot isolationやdecodeの問題ではなく、support packageを作る前段のcoverage failureである。

この時点で、本来一般化すべきだった失敗型は次である。

```text
- coordinate roleを含む代数IR
- multi-variable projection
- guard / branch付きtarget projection
- determinant / oriented area型の双線形構造
- dot / Gram型の構造
- coordinate signature
- tower / extension構造
- support packageを作るための汎用target projection機構
```

しかし実際には、最終的に次のような狭い内部stressに落ちた。

```text
- T^2 - c
- x^2 = c, T + ax - b = 0
- u^2 = a, v^2 = b, D = u^2 + v^2
```

これは、実問題5問を直接Planに入れないという正しい方針を、

```text
実問題5問由来の代数構造も十分に入れない
```

という誤った結果にした失敗である。

---

## 2.7 「実問題5問を入れない」と「実問題構造を入れない」を混同した

実問題5問をPlanに直接入れると、Agentが5問だけを通すhackをする危険があった。  
これは正しい懸念だった。

しかし、正しい対応は次である。

```text
問題名・式・答えは使わない。
しかし、5問で必要になった代数構造を一般化して必須stressにする。
```

実際には、これが不十分だった。

その結果、

```text
テスト過適合は避けたが、現実の代数構造も避けてしまった
```

という状態になった。

---

## 2.8 Gate依存を防ぐつもりで、別のGate依存を作った

当初から、

```text
Agentがacceptance gateを通すことだけを目的にしてはいけない
```

という懸念があった。

そのため、reviewer promptやreview_summaryを整備した。

しかし結果的に、次のような新しいgate依存が生まれた。

```text
- reviewer promptを使っている
- review_summary.yamlがある
- phase_closable=true
- mandatory scans classified
- evidenceが整っている
```

これらが整っていることを、実装が正しいことの強い証拠として扱いすぎた。

一般化すると、次の失敗である。

```text
古いacceptance gate依存を、新しいGuardian/reviewer protocol依存に置き換えただけだった。
```

---

## 2.9 Reviewerがアルゴリズム十分性ではなく、証拠整合性を見すぎた

reviewer promptでは、多くの場合、次を見ていた。

```text
- fallbackがないか
- old engineがないか
- hashがbindされているか
- promptを使ったか
- evidenceがあるか
- claim ceilingを守っているか
```

これらは必要である。  
しかし、十分ではない。

本来は、reviewerに次を強く見させるべきだった。

```text
この実装は、代数IRからtarget候補を汎用的に列挙するアルゴリズムになっているか？
部分対応をきれいに文書化しているだけではないか？
unsupportedがアルゴリズム欠陥を隠していないか？
このまま進めると、本当に汎用DTPK theorem coreに到達するか？
```

この観点が弱かったため、

```text
過大claimはしていないが、目的のsolverにもなっていない
```

状態をPASSできてしまった。

---

## 2.10 「過大claimしていない」ことを「正しい進捗」と誤認した

多くのphaseで、次のような理由で肯定的に評価した。

```text
- theorem-core completionはまだ主張していない
- speed claimはしていない
- R9/R10はpreflightだけだと書いている
```

これは事実としては正しい。

しかし、それは単に

```text
嘘の完成宣言をしていない
```

というだけである。

本来必要なのは、

```text
完成へ向かう実装が本当に進んでいる
```

ことである。

「まだ完成と言っていない」ことは、「完成に近づいている」ことの証拠ではない。

---

## 2.11 R9/R10のpreflightを進捗として過大評価した

R9は外部検証の準備であって、外部検証そのものではなかった。  
R10は性能測定の準備であって、性能測定そのものではなかった。

つまり、R9/R10完了は、

```text
これから検証する準備ができた
```

にすぎない。

それを、計画完了やアルゴリズム完成に近いものとして扱うのは誤りだった。

一般化すると、次の失敗である。

```text
preflightを、実質的な検証や完成と混同した。
```

---

## 2.12 support packageが出ることを重視しすぎた

実問題5問では、support packageが1つも出なかった。

そのため、support packageを出すことを強く重視した。

これは必要だった。

しかし、途中から、

```text
狭いsliceならsupport packageが出る
```

ことを強く評価しすぎた。

本来必要だったのは、

```text
広い代数IRに対して、target candidate coverを作る
```

ことである。

support packageが出ること自体ではなく、

```text
どの範囲で、どのアルゴリズムにより出るのか
```

が重要だった。

---

## 2.13 candidate isolation未実装を見逃した

R7の時点では、

```text
support polynomialは出る
root isolationは空
decoded candidatesはplaceholder的表現
```

という状態だった。

これは、target candidate solverとして基本的な未完成だった。

後でR7Rで修正したが、本来はR7をPASSする前に止めるべきだった。

一般化すると、次の失敗である。

```text
support polynomialの構成と、解候補列挙の完成を混同した。
```

---

## 2.14 heavy fallbackを見つけた後の停止判断が遅れた

一度、P6〜P9で `ExactEliminationBackbone` やBuchberger型target eliminationが広く使われる危険があった。

これは、NoCompletenessFallbackの方針から見て重大な問題だった。

本来ならその時点で、

```text
P6以降はFAIL
P5まで戻すべき
PlanDefect / AlgorithmDefectとして止めるべき
```

だった。

しかし最初は、

```text
coverage backboneとしては許容できるかもしれない
P10/P12で見るべき
```

という甘い評価をした。

これは重大な判断ミスだった。

---

## 2.15 heavy fallbackを潰した結果、今度は部分対応へ逃げた

P5 reset後は、heavy fallbackを強く禁止した。

これは正しい方向だった。

しかし、その結果、

```text
広く解くのは危険だから、狭いsliceだけを解く
```

になった。

つまり、失敗は次のように移動した。

```text
初期の失敗:
  部分対応の寄せ集め

次の失敗:
  heavy generic fallback

さらに次の失敗:
  heavy fallbackを避けるために、また部分対応へ戻る
```

根本的には、

```text
heavy fallback禁止
部分対応禁止
汎用代数IR solver
```

を同時に満たせなかったことが原因である。

---

## 2.16 CoverageMatrixを逃げ道として使った

CoverageMatrixは、本来、theorem claimを安全に限定するための仕組みである。

狭いsliceを中間実装単位として使うこと自体は許される。  
しかし、それは最終目標ではない。

今回の失敗は、CoverageMatrixを、

```text
この外はout_of_supported_sliceでよい
```

という逃げ道として使いすぎたことにある。

本来は、

```text
汎用DTPKへ向かう途中の検証単位
```

として使うべきだった。

---

## 2.17 feature certificateを「planner情報」ではなく「対応済みパターンの入場券」にしてしまった

代数的feature certificate自体は良い考えである。

例えば、

```text
- このblockはtarget-linear
- このblockはtower構造
- このblockはsparse resultant向き
- このblockはtarget action向き
```

という情報を証明し、plannerが処理を選ぶのは妥当である。

しかし今回の実装では、feature certificateが、

```text
このfeatureがあるものだけ解く
ないものはslice外
```

という入場券になってしまった。

本来は、feature certificateは

```text
汎用DTPKアルゴリズムの中で、どの効率的な処理を選ぶかを決める材料
```

であるべきだった。

---

## 2.18 DTPKを単一main algorithmではなく、projectorの寄せ集めとして扱った

DTPKは、本来、単一のproduction main algorithmである。

```text
TargetProjectionDAG
Deterministic planner
Projectors
Global assembly
Candidate isolation
Fiber classification
Certificate finalization
```

が一体化して動く必要がある。

しかしレビューでは、

```text
このprojectorがある
このgateがある
このpackageがある
```

という部品単位の評価に偏った。

本来見るべきだったのは、

```text
全体としてtarget候補列挙アルゴリズムが成立しているか
```

である。

部品があるだけではDTPKではない。

---

## 2.19 DAGやcertificateが飾りになる危険を十分に潰せなかった

過去には、次のような危険があった。

```text
- DAGはあるが、実行時に別のclosureを取り直す
- specialized projectorはあるが、generic quotient/actionのtraceにすぎない
- certificateはあるが、実際の計算を制約していない
```

本来は、最初のBase Specから、

```text
DAGやcertificateを削除・改変したらrunが失敗すること
```

を必須にすべきだった。

途中からsemantic deletion challengeを入れたが、遅かった。

---

## 2.20 admissionが「解けること」を保証していなかった

実問題5問の診断では、SparseLocalがlight admissionではselectedされるが、heavy build_handleで失敗するケースがあった。

これは一般化すると、

```text
admissionが、support-producing preexecution planの存在を保証していない
```

という失敗である。

plannerがprojectorを選ぶなら、少なくとも、

```text
そのprojectorがsupport packageを作る実行計画を構成できる
```

ところまで保証する必要がある。

---

## 2.21 reviewer promptだけではAgentの失敗を防げなかった

phaseごとのreviewer prompt、red-team reviewer、review_summaryなどを追加した。

しかし、それでも失敗した。

理由は、reviewer prompt以前に、Base Spec / Plan自体が部分対応やpreflight完了を許していたからである。

一般化すると、

```text
reviewerは、照合先の仕様が間違っていれば、その間違った仕様に沿ってPASSする。
```

したがって、reviewer promptの改善だけでは足りない。  
Base Specそのものが、最終的な汎用代数IR solverを十分に定義していなければならない。

---

## 2.22 active authority問題を本質原因として過大評価した

一時期、失敗原因として、

```text
ACTIVE_CONTEXTが古い
active authorityがv6.2のまま
```

を強く見た。

しかし、それは本質ではなかった。

本質は、

```text
Base Spec / Planそのものに重大な穴があった
```

ことである。

ACTIVE_CONTEXTの更新漏れは文書管理上の問題ではあるが、アルゴリズム失敗の根本原因ではない。

---

## 2.23 docsが整っていることを信頼しすぎた

Guardian docs、evidence、review archive、hash、footerが整うと、見た目はかなり信頼できる。

しかし、文書が整っていることは、アルゴリズムが正しいことを意味しない。

今回の失敗は、

```text
文書とreviewは整っているが、目的の汎用solverには届いていない
```

というものだった。

これは、かなり危険な失敗である。

---

## 2.24 内部stressが実問題構造を代表していなかった

内部stressは、実問題の名前や答えを使わずに、実問題由来の代数構造を一般化したものであるべきだった。

しかし実際には、A0 / Metric-B0の小さいstressに寄った。

これは、

```text
テストhackは避けたが、現実性も失った
```

状態である。

---

## 2.25 Performance-firstを最後に測るものとして扱いすぎた

性能主張は、もちろん測定が必要である。

しかしperformance-firstは、最後に測るだけではない。  
アルゴリズム設計の段階から、次を見なければならない。

```text
- どの変数を消さずに済むのか
- どの行列を小さくするのか
- target degreeをどう下げるのか
- spurious candidatesをどう抑えるのか
- fiber classificationをどうbatch化するのか
- checker costをどう抑えるのか
```

今回のR10は、性能測定preflightであり、性能設計そのものではなかった。

---

## 2.26 CoreTheoremGate / PerformanceClaimGateが通っていないのに、計画完了のように扱った

R9/R10後も、CoreTheoremGateやPerformanceClaimGateは通っていなかった。

つまり、

```text
theorem core completion は未承認
performance claim も未承認
```

だった。

それにもかかわらず、計画完了に近いような扱いをしたのは誤りだった。

---

## 2.27 汎用代数アルゴリズムの中身をBase Specで具体化できなかった

Base Specに、

```text
汎用代数IR target候補列挙アルゴリズムとは具体的に何か
```

を十分に書けなかった。

本来は、少なくとも次を明示する必要があった。

```text
- target-compatible projection
- multi-variable projection DAG
- guard/branch-aware saturation
- target action / quotient handle
- Krylov / trace / norm / resultant
- target relation tightening
- global target eliminant assembly
- candidate isolation
- fiber / guard validation
```

これらをどう統合するかを定義できなかったため、実装がprojector portfolioやslice列挙に流れた。

---

## 2.28 「汎用」と「全部解く」を混同し、その反動で狭くしすぎた

汎用solverとは、すべての有限代数問題を解くという意味ではない。

しかし、実用的に出てくる広い代数IR構造を、共通の代数原理で処理する必要がある。

私は、

```text
全finite IRを解くのは過大claim
```

という正しい認識から、

```text
では狭いsliceだけでよい
```

へ行きすぎた。

正しくは、

```text
全finite IRではないが、実問題由来の主要代数構造は共通アルゴリズムで扱う
```

だった。

---

## 2.29 小手先のPlan修正を繰り返した

何度もBase Spec / Plan / reviewer promptを修正したが、その多くは、

```text
前回失敗した具体例を禁止する
```

方向だった。

たとえば、

```text
- squared_distance-only禁止
- bivariate-only禁止
- coordinate chart rejection禁止
- generic fallback禁止
- placeholder candidate禁止
```

である。

これらは必要だったが、十分ではない。

本来禁止すべきだったのは、

```text
本来の目的を満たさない局所対応を完成扱いすること
実装都合でscopeを狭めること
supportを作れない構造をunsupportedへ逃がすこと
preflightやgate整備を完成と見なすこと
```

である。

---

## 2.30 PlanDefect / AlgorithmDefectで止めるべき場面で止めなかった

以下の場面では、本来PlanDefect / AlgorithmDefectとして止めるべきだった。

```text
- heavy fallbackが広く入り始めた時
- declared sliceが狭すぎるとわかった時
- R7でcandidate isolationがplaceholderだった時
- R9/R10がpreflightでしかないのに計画完了扱いされた時
- 代数IR solverなのに幾何family的sliceへ寄った時
```

しかし、私は何度も次phaseへ進める判断をした。

これはレビュー者としての失敗である。

---

## 2.31 「計画通り」と「目的達成」を混同した

私は何度も、

```text
phaseとしては計画通り
概ねPASS
ここまでは良い
```

と評価した。

しかし、本来見るべきだったのは、

```text
この計画自体が、最初の目的と一致しているか
```

である。

計画自体が狭いslice solverへズレているなら、

```text
計画通り
```

と言っても意味がない。

---

## 2.32 実装があることと研究アルゴリズムが成立していることを混同した

Rustのコードがあること、testが通ること、reviewがあることと、研究として意図したアルゴリズムが成立していることは別である。

DTPKは研究アルゴリズムである。

したがって、実装レビューでは、

```text
- 型がある
- 関数がある
- hashがある
- testが通る
```

だけでは足りない。

見るべきだったのは、

```text
- この計算は本当にtarget-direct projectionとして意味があるか
- 実問題由来の代数構造に耐えるか
- 計算量はperformance-firstとして妥当か
```

である。

---

## 2.33 外部ライブラリ検討後に、中核アルゴリズムを詰め切れなかった

途中で、外部CAS、Groebner、homotopy、QE/CAD、optimizationなどで代用できるかを検討した。

結論として、これらはbaselineやdebug oracleにはなり得るが、GeoSolverのproduction certified pathにはならない、という方向だった。

しかしその後、

```text
では外部CASでも汎用Groebnerでもない、中核アルゴリズムは具体的に何か
```

を十分に詰められなかった。

その隙間を、ある時はgeneric fallbackが埋め、別の時はnarrow sliceが埋めた。

---

## 2.34 入力scopeとtheorem claimの分離に失敗した

本来は、次を同時に満たす必要があった。

```text
- 入力としての代数IRは広い
- solver coreは広い構造を処理する
- theorem claimは正確に限定する
```

しかし実際には、

```text
広く解くように見せてfallbackに寄る
```

か、

```text
狭いslice外はunsupportedにする
```

かの二択になった。

正しくは、

```text
広い代数IRを受け、
DTPKの共通代数機構で処理し、
できない場合はAlgorithmDefectとして扱うべき構造と、
本当にscope外の構造を厳密に分ける
```

だった。

---

## 2.35 future workを安易に増やした

CoverageMatrix外をfuture workにすることは、場合によっては必要である。

しかし、実問題5問に必要な主要構造までfuture workに回すと、solver coreの価値がなくなる。

今回、

```text
今後sliceを増やせばよい
```

という説明をしたが、これは不適切だった。

本来は、

```text
今のcoreに必要な代数構造
```

と、

```text
将来拡張でよい構造
```

を区別しなければならなかった。

---

## 2.36 検証が後追いになりすぎた

多くの場合、ユーザーが違和感を示した後に、

```text
確かに問題です
修正しましょう
```

となった。

本来は、私のレビュー段階で先に検出すべきだった。

これは、レビュー者としての失敗である。

---

# 3. すべての失敗原因を一般化したまとめ

上記を一般化すると、失敗原因は次の15個に集約できる。

```text
1. 最初の目的を、phase完了・gate通過・文書整備にすり替えた。
2. 代数IR solverであるという前提を何度も見失った。
3. 幾何family対応のような考え方をsolver coreに持ち込んだ。
4. 部分対応を防ぐはずが、別の形の部分対応を許した。
5. NoFallbackとnarrow scopeを混同した。
6. heavy fallback禁止と汎用性維持を同時に満たす設計を作れなかった。
7. 実問題5問由来の代数構造を、十分に一般化stressへ変換できなかった。
8. Reviewer promptが証拠整合性に偏り、アルゴリズム十分性を見切れなかった。
9. Base Specが、最終的にあるべき汎用代数アルゴリズムを十分に定義できなかった。
10. Gateやpreflightを、実質的な検証・完成と混同した。
11. Candidate isolationやsupport coverageなどの基本未実装を途中で見逃した。
12. Heavy fallbackを見つけた後の停止判断が遅れた。
13. CoverageMatrixを、中間開発単位ではなく逃げ道として使った。
14. PlanDefect / AlgorithmDefectとして止めるべき場面で止めなかった。
15. レビューが、目的達成ではなく局所的なphase整合性に流れた。
```

---

# 4. 今後絶対に必要な方針

今後のBase Spec / Planでは、最低限、次を固定しなければならない。

```text
1. 入力は幾何ではなく代数IRである。
2. solver coreは幾何family名でdispatchしない。
3. target candidate coverを作る汎用的な代数機構を定義する。
4. heavy fallbackは禁止する。
5. narrow slice / unsupported乱用による部分対応逃げも禁止する。
6. 実問題5問由来の代数構造を、問題名や答えなしで一般化stressにする。
7. そのstressはblockerではなくsupport-producing success caseにする。
8. failure時はslice外扱いではなく、AlgorithmDefect / PlanDefectとして止める。
9. CoreTheoremGate / PerformanceClaimGateを実際に通すまで完成と言わない。
10. preflight完了を、実検証完了や性能確認と混同しない。
```

---

# 5. 次のBase Spec / Planで禁止すべき一般パターン

次のような実装・計画・レビューは、名前や形を変えても禁止する必要がある。

```text
- 実装しやすい形だけを対応範囲として切り出す。
- unsupportedを正直に返すことを、完成に近いものとして扱う。
- gateやreviewが整っていることを、アルゴリズム完成の証拠とする。
- feature certificateを対応済みパターンの入場券として使う。
- target-directという名前でheavy generic eliminationを隠す。
- NoFallbackの名の下でnarrow scopeへ逃げる。
- 内部toy stressだけで現実の代数構造を代表したことにする。
- preflightを実検証と混同する。
- CoreTheoremGate / PerformanceClaimGateが通っていないのに完成扱いする。
- 幾何DSL層と代数IR solver層の責務を混ぜる。
```

---

# 6. 最終結論

今回までの流れで起きたことは、単なる実装バグではない。

根本的には、

```text
代数IRからtarget候補を汎用的・高速に列挙するDTPK solver core
```

を作るべきだったのに、

```text
一時はheavy fallbackへ寄り、
それを潰した後は狭いdeclared slice solverへ寄り、
最終的には文書・review・preflightが整った小さいsolverになった
```

という失敗である。

これは、最初から避けるべきだった

```text
部分対応で完成したように見せる
```

失敗の再発である。

今後は、A0 / Metric-B0 の次のsliceを単に足すのではなく、まずBase Spec / Planを根本から作り直し、

```text
heavy fallback禁止
部分対応禁止
汎用代数IR target候補列挙
```

を同時に満たすアルゴリズムを定義しなければならない。

