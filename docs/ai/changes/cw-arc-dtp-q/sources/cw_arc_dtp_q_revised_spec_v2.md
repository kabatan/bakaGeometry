# CW-ARC-DTP-Q 改訂仕様書 v2

**正式名:** Certificate-Window Learning Adaptive Residual-Certificate Direct Target Projection over Rational Polynomial Systems  
**略称:** CW-ARC-DTP-Q  
**対象:** `geosolver-core` の中核 target 値 solver  
**文書種別:** 理想アルゴリズムを固定する自己完結仕様書（改訂版 v2）  
**非対象:** 実装順序、テスト計画、benchmark 設計、folder 構成、UI、幾何 DSL lowering の詳細

**v2 改訂要旨:** guard 証明を独立 object として明示し、空 admissible 集合の status、proof 探索の公平性、modular proof construction、`CertifiedExactTargetImage` への標準 pipeline を追加した。

---

## 0. この仕様の要旨

CW-ARC-DTP-Q は、有理数係数多項式系

\[
F=\{F_1,\dots,F_m\}\subset\mathbb Q[X_1,\dots,X_n,T]
\]

から、全座標解を構成せず、target 変数 \(T\) が満たす一変数多項式

\[
0\neq S(T)\in\mathbb Q[T]
\]

を証明付きで求めるアルゴリズムである。

中核方針は次の 1 文に尽きる。

> 有限体・特殊化・Krylov・resultant はすべて候補生成にだけ使い、採用は元の有理数係数系に対する exact certificate が存在する場合に限る。

したがって、solver が target cover の成功を返すための唯一の本質条件は、元の系または証明付きで同値変形された系に対して、次のいずれかを exact に証明できることである。

\[
S(T)\in I,
\qquad
S(T)^a\in I,
\qquad
D^eS(T)^a\in I,
\]

ここで

\[
I=\langle F_1,\dots,F_m\rangle,
\]

\(D\) は admissible fiber 上で非ゼロであることが独立した `GuardCertificate` により証明された guard 積であり、\(a\ge 1\), \(e\ge 0\) は整数である。

この仕様では、局所 projection generator や full coordinate quotient を主対象にしない。主対象は常に **target support polynomial とその証明書** である。

標準の成功段階は次の順に分かれる。

```text
1. CertifiedCandidateCover
   真の target 値は S(T) の実根候補に含まれる。

2. CertifiedExactTargetImage
   S(T) の各実根について real admissible fiber の有無を exact に分類し、余分な実根を除いた値集合を返す。

3. CertifiedEmptyAdmissibleSet
   入力条件を満たす admissible 解が存在しないことを exact に返す。
```

`CertifiedCandidateCover` は「値の候補集合」を返す段階であり、余分な実根を含んでよい。`CertifiedExactTargetImage` は、その後段として exact real fiber classification が完了した場合にだけ返してよい。

## 1. 仕様記法と拘束力

本仕様で使う語の意味を固定する。

| 表現 | 意味 |
|---|---|
| **しなければならない** | 実装が満たす必須条件 |
| **してはならない** | 実装が行ってはならない禁止事項 |
| **してよい** | 正しさを壊さない範囲で許可される選択 |
| **候補** | exact certificate が付くまで採用してはならない object |
| **証明書** | 検証器が有理数演算だけで再検証できる object |
| **admissible 解** | 入力等式と semantic guard をすべて満たす解 |
| **support polynomial** | 真の target 値集合を根集合で覆う非零一変数多項式 |

全ての確率的・有限体的・特殊化的な計算結果は、明示的に `Candidate` として扱わなければならない。`Candidate` は成功出力ではない。

---

## 2. 数学的問題設定

### 2.1 基本環

\[
R=\mathbb Q[X_1,\dots,X_n,T]
\]

とする。入力方程式集合を

\[
F=(F_1,
\dots,F_m)
\]

とし、入力 ideal を

\[
I=\langle F_1,
\dots,F_m\rangle\subset R
\]

とする。

target 変数は \(T\) であり、\(X=(X_1,
\dots,X_n)\) は target 以外の全変数を表す。

### 2.2 guard、admissibility、saturation

不等式、非ゼロ条件、分母非ゼロ条件、幾何 lowering に由来する semantic condition は、等式系とは別に provenance を持つ。

ただし、semantic guard はそれ自体では代数的証明ではない。guard を guarded radical certificate に使うには、各 guard polynomial \(D_j\) について、admissible fiber 上で

\[
D_j\neq 0
\]

であることを `GuardCertificate` として検証可能に示さなければならない。

admissible fiber 上で非ゼロであることが証明された polynomial の集合を

\[
\mathcal D=\{D_1,
\dots,D_s\}\subset R
\]

とし、

\[
D=\prod_{j=1}^sD_j
\]

を certified guard 積とする。\(D\) に含めてよい因子は、`GuardCertificate` を持つものだけである。

CW-ARC-DTP-Q の candidate cover は、必要に応じて guarded ideal

\[
I:D^\infty=\{h\in R\mid \exists e\ge 0,
D^eh\in I\}
\]

に対する target relation を扱ってよい。ただし、guard を使う場合は、\(D\neq 0\) が admissible fiber 上で保証されることを証明書に含めなければならない。

### 2.3 complex cover と real image の区別

`CertifiedCandidateCover` は、代数的な target support polynomial を返す段階である。これは原則として、等式系と certified nonzero guard から従う代数的 cover である。

`CertifiedExactTargetImage` は、実数解を対象にする後段である。各実根 \(\alpha\) について、次の実閉体上の文の真偽を exact に分類しなければならない。

\[
\exists X\in\mathbb R^n
\quad
F_1(X,\alpha)=\cdots=F_m(X,\alpha)=0
\quad\land\quad
\text{semantic guards are satisfied}.
\]

この分類が全ての実根について完了していない場合、solver は `CertifiedExactTargetImage` を返してはならない。

### 2.4 目標

target cover の成功出力は、非零多項式

\[
S(T)\in\mathbb Q[T]
\]

と証明書 \(C\) の組である。

\(C\) は次を検証可能に示さなければならない。

\[
\forall (x,t)\in V_{adm},\quad S(t)=0.
\]

ここで \(V_{adm}\) は入力の等式と semantic guard を満たす admissible 解集合である。

admissible 解集合が空であることを exact に証明できる場合は、任意の \(S\) を cover として返すのではなく、`CertifiedEmptyAdmissibleSet` を返さなければならない。

この仕様の標準成功 status は `CertifiedCandidateCover` である。これは根集合に余分な候補を含んでよい。余分な実根を除く exact real image 判定は別段階であり、本仕様の中核採用条件ではない。

## 3. 禁止事項

CW-ARC-DTP-Q の production path は次をしてはならない。

1. 全座標解リストを作る。
2. full coordinate RUR を作る。
3. 全座標 lex parametrization を作ってから \(T\) を読む。
4. 幾何名、問題名、fixture 名、期待答え、公式解で分岐する。
5. 有限体計算、数値計算、特殊化結果、Krylov recurrence を exact proof なしに採用する。
6. 弱い局所 projection message から `CertifiedNonFiniteTargetImage` を返す。
7. planner に明示されていない hidden fallback を呼ぶ。
8. `Unsupported` を通常失敗として返す。

許される重い処理は、target/separator relation の構成、fixed target membership 証明、localized Schur repair、最後の target-only exact elimination に限る。

---

## 4. 中核設計原則

CW-ARC-DTP-Q は次の 5 原則を満たさなければならない。

### 4.1 採用ゲートは 1 つだけ

候補 \(S(T)\) を採用する唯一のゲートは exact verifier である。候補が何個の有限体で一致しても、何個の slice で再現しても、採用条件にはならない。

### 4.2 候補生成と証明を分離する

候補生成器は不完全でもよい。誤った候補を返してもよい。ただし、その候補が成功出力になることはない。

証明器は完全に保守的でなければならない。証明できない object は不採用である。

### 4.3 主 object は projection generator ではなく certificate window

局所 block から 1 本の relation generator を返す設計は主経路ではない。主経路の object は、target 候補 \(S(T)\) と、その \(S(T)\) を証明するための有限 support window である。

### 4.4 matrix 表現は隠す

残差計算は、Schur 行列を物理的に作っても、疎 row basis を使っても、Krylov sketch を使ってもよい。ただし外部契約は常に

\[
\rho_W(v)=v\bmod\operatorname{col}(M_W)
\]

という residual oracle でなければならない。

### 4.5 失敗は fail-closed

証明できない場合は `NoVerifiedTargetCertificate` または resource/certificate failure を返す。非有限性や空性を推測で返してはならない。

---

## 5. 最小データモデル

この節は実装の進め方ではなく、アルゴリズムが保持すべき数学的 object の contract である。

### 5.1 入力 object

```rust
pub struct TargetProblemQ {
    pub equations: Vec<PolynomialQ>,
    pub variables: Vec<Variable>,
    pub target: Variable,
    pub semantic_guards: Vec<GuardRecord>,
}
```

`GuardRecord` は、入力側の意味論として与えられた guard の記録である。

```rust
pub struct GuardRecord {
    pub polynomial: PolynomialQ,
    pub kind: GuardKind,
    pub provenance: GuardProvenance,
}

pub enum GuardKind {
    NonZero,
    Positive,
    Negative,
    NonNegative,
    NonPositive,
    OtherSemanticCondition,
}
```

必須 invariant:

```text
- equations は Q 係数多項式である。
- target は variables に含まれる。
- semantic_guards は candidate 採用には使わず、guard certificate と real fiber 判定にだけ使う。
- GuardKind::NonZero 以外の guard を guarded radical の D 因子として使うには、別途 nonzero を導く GuardCertificate が必要である。
- variable role は provenance であり、algorithmic dispatch に使わない。
```

### 5.2 証明付き圧縮系

```rust
pub struct CertifiedSystemQ {
    pub equations: Vec<PolynomialQ>,
    pub target: Variable,
    pub guard_certificates: Vec<GuardCertificate>,
    pub replay: CompressionReplayCertificate,
}
```

`CertifiedSystemQ` は入力系から certificate-preserving rewrite だけで得られなければならない。

許される rewrite:

```text
- 定義式 y-p=0 の代入
- 非ゼロ性が証明された pivot による affine elimination
- 明示 guard による saturation
- primitive normalization
- zero equation removal
```

禁止される rewrite:

```text
- 因子を勝手に選ぶ
- branch を期待答えで選ぶ
- 幾何名に基づく公式変形
- replay 不能な外部 CAS simplification
```

### 5.3 guard certificate

`GuardCertificate` は、ある guard polynomial が admissible fiber 上で非ゼロであることを verifier が再検証するための object である。

```rust
pub enum GuardCertificate {
    InputSemanticNonzero {
        guard: PolynomialQ,
        record: GuardRecord,
    },
    AlgebraicNonvanishing {
        guard: PolynomialQ,
        certificate: NullstellensatzCertificate,
    },
    RealAdmissibleNonvanishing {
        guard: PolynomialQ,
        certificate: RealInfeasibilityCertificate,
    },
    DerivedProduct {
        product: PolynomialQ,
        factors: Vec<GuardCertificate>,
        identity: ExactIdentity, // product = Π factor.guard
    },
}
```

各 variant の意味は次である。

```text
- InputSemanticNonzero:
  入力の semantic_guards に同一 polynomial の NonZero record が存在することを確認する。

- AlgebraicNonvanishing:
  V(I) 上で guard=0 が不可能であることを、例えば 1 ∈ <F, guard> の Nullstellensatz certificate で示す。

- RealAdmissibleNonvanishing:
  実 admissible 解で guard=0 が不可能であることを、実閉体判定または Positivstellensatz 型 certificate で示す。

- DerivedProduct:
  複数 guard の積を作るための replay であり、新しい仮定ではない。
```

`GuardCertificate` は candidate 採用の補助情報ではなく、`GuardedRadicalMembership` の soundness に必要な証明書である。

### 5.4 target certificate

```rust
pub enum TargetCertificate {
    IdealMembership {
        support: UniPolynomialQ,
        multipliers: Vec<PolynomialQ>,
        identity: ExactIdentity,
    },
    RadicalMembership {
        support: UniPolynomialQ,
        power: usize,
        multipliers: Vec<PolynomialQ>,
        identity: ExactIdentity, // support^power = Σ q_i F_i
    },
    GuardedRadicalMembership {
        support: UniPolynomialQ,
        support_power: usize,
        guard_power: usize,
        guard_product: PolynomialQ,
        guard_certificates: Vec<GuardCertificate>,
        multipliers: Vec<PolynomialQ>,
        identity: ExactIdentity, // D^e support^a = Σ q_i F_i
    },
    CompositeCover {
        support: UniPolynomialQ,
        children: Vec<TargetCertificate>,
        rule: CompositeRule,
    },
}
```

`CompositeRule` は次のどちらかだけである。

```rust
pub enum CompositeRule {
    SameIdealGcd,
    ComponentUnionLcm,
}
```

### 5.5 solver-level certificate

`TargetCertificate` は target cover だけを表す。空性、no eliminant、exact image は別種の証明であるため、top-level では次の型で包む。

```rust
pub enum SolverCertificate {
    TargetCover(TargetCertificate),
    ExactTargetImage(ExactTargetImageCertificate),
    EmptyAdmissibleSet(EmptyAdmissibleSetCertificate),
    NoNonzeroTargetEliminant(NoTargetEliminantCertificate),
}
```

空 admissible 集合を cover 多項式で偽装してはならない。

```rust
pub enum EmptyAdmissibleSetCertificate {
    AlgebraicInfeasibility {
        multipliers: Vec<PolynomialQ>,
        identity: ExactIdentity, // 1 = Σ q_i F_i
    },
    GuardedAlgebraicInfeasibility {
        guard_product: PolynomialQ,
        guard_power: usize,
        guard_certificates: Vec<GuardCertificate>,
        multipliers: Vec<PolynomialQ>,
        identity: ExactIdentity, // D^e = Σ q_i F_i
    },
    RealInfeasibility {
        certificate: RealInfeasibilityCertificate,
    },
}
```

`NoTargetEliminantCertificate` は、complete fallback が target elimination ideal の零性を exact に証明したことを表す。

```rust
pub struct NoTargetEliminantCertificate {
    pub saturated_ideal_description: SaturatedIdealCertificate,
    pub elimination_certificate: EliminationZeroCertificate,
    pub guard_certificates: Vec<GuardCertificate>,
}
```

### 5.6 exact target image certificate

```rust
pub struct ExactTargetImageCertificate {
    pub cover: TargetCertificate,
    pub squarefree_support: UniPolynomialQ,
    pub root_classifications: Vec<RealRootFiberCertificate>,
}

pub struct CertifiedExactTargetImage {
    pub support: UniPolynomialQ,
    pub squarefree_support: UniPolynomialQ,
    pub values: Vec<AlgebraicRealRoot>,
    pub rejected_roots: Vec<AlgebraicRealRoot>,
    pub certificate: ExactTargetImageCertificate,
}

pub enum RealRootFiberCertificate {
    Nonempty {
        root: AlgebraicRealRoot,
        certificate: RealFiberNonemptyCertificate,
    },
    Empty {
        root: AlgebraicRealRoot,
        certificate: RealFiberEmptyCertificate,
    },
}
```

`ExactTargetImageCertificate` は、`squarefree_support` の全ての実根を `Nonempty` または `Empty` に分類していなければならない。一部だけ分類した object は trace であり、成功 certificate ではない。

`CertifiedExactTargetImage.values` には `Nonempty` に分類された根だけを入れる。`rejected_roots` には `Empty` に分類された余分な実根を入れる。

## 6. 証明書言語

### 6.1 Ideal membership certificate

\(S(T)\) が採用される最も単純な条件は、ある \(q_i\in R\) が存在して

\[
S(T)=\sum_{i=1}^m q_iF_i
\]

が exact に成り立つことである。

証明書は \(q_i\) の sparse 表現と、恒等式

\[
S(T)-\sum_iq_iF_i=0
\]

を有理数演算だけで再検証できる情報を含まなければならない。

### 6.2 Radical certificate

ある \(a\ge 1\) について

\[
S(T)^a=\sum_iq_iF_i
\]

が成り立てば、任意の解で \(S(T)=0\) である。したがって \(S\) は candidate cover として採用してよい。

### 6.3 Guarded radical certificate

ある \(a\ge 1\), \(e\ge 0\) について

\[
D^eS(T)^a=\sum_iq_iF_i
\]

が成り立ち、かつ admissible fiber 上で \(D\neq 0\) が保証されるなら、任意の admissible 解で \(S(T)=0\) である。

`GuardedRadicalMembership` の verifier は次を全て確認しなければならない。

```text
- guard_product が guard_certificates 内の guard の積と exact に一致する。
- 各 GuardCertificate が入力 problem と replay 後 system に対して valid である。
- identity D^e S^a = Σ q_i F_i が Q 上の sparse polynomial identity として exact に 0 になる。
```

semantic guard の存在だけを理由に `GuardedRadicalMembership` を採用してはならない。guard が `InputSemanticNonzero` として使われる場合でも、入力の `semantic_guards` に同一 polynomial の `GuardKind::NonZero` record が存在することを verifier が確認しなければならない。

### 6.4 Composite certificate

同一 ideal から得た verified cover

\[
S_1,
\dots,S_k
\]

は

\[
S_{gcd}=\gcd(S_1,
\dots,S_k)
\]

に refinement してよい。証明は「任意の admissible target は全ての \(S_i\) の根であるため、共通根集合を定める \(S_{gcd}\) の根でもある」という root-set inclusion である。

component union の場合だけ

\[
S_{lcm}=\operatorname{lcm}(S_1,
\dots,S_k)
\]

を使う。これは各 component の target 値集合の和集合を覆うためである。

### 6.5 Empty admissible set certificate

admissible 解集合が空であることを証明できる場合、solver は `CertifiedEmptyAdmissibleSet` を返してよい。

代数的空性は、例えば

\[
1=\sum_iq_iF_i
\]

で証明される。

guarded algebraic infeasibility は、certified guard product \(D\) について

\[
D^e=\sum_iq_iF_i
\]

を示し、さらに admissible 解で \(D\neq0\) を guard certificate で示すことで証明される。admissible 解では左辺が非零、右辺が 0 になるため矛盾する。

実数 admissible 解の空性は、実閉体上の infeasibility certificate または Positivstellensatz 型 certificate で証明される。

空性が証明された場合、`S(T)=1` の cover として返してはならない。API status は必ず `CertifiedEmptyAdmissibleSet` でなければならない。

### 6.6 No nonzero target eliminant certificate

\[
(I:D^\infty)\cap\mathbb Q[T]=\{0\}
\]

を返すには、complete target elimination fallback が elimination ideal の零性を exact に証明しなければならない。

この statement は代数的 statement であり、real admissible target image が非有限であることを意味しない。real 非有限性を返すには、別途 real dimension / feasibility certificate が必要である。

## 7. Certificate Window

### 7.1 定義

certificate window \(W\) は次の有限データである。

\[
W=(d,
B_1,
\dots,B_m,
C)
\]

ここで

- \(d\) は target degree bound。
- \(B_i\subset\operatorname{Mon}(X,T)\) は multiplier \(q_i\) の monomial support。
- \(C\subset\operatorname{Mon}(X,T)\) は係数比較に使う row monomial set。

\(W\) は row-closed でなければならない。

\[
C=\operatorname{Supp}\{1,T,
\dots,T^d\}\cup
\bigcup_{i=1}^m\operatorname{Supp}(B_iF_i).
\]

ここで

\[
\operatorname{Supp}(B_iF_i)
=
\bigcup_{b\in B_i}\operatorname{Supp}(bF_i).
\]

### 7.2 membership matrix

各 \((i,b)\), \(b\in B_i\), に対し、列 vector

\[
\operatorname{vec}_C(bF_i)
\]

を作る。これらを並べた行列を

\[
M_W
\]

とする。

target powers の列を

\[
N_d=[\operatorname{vec}_C(1),
\operatorname{vec}_C(T),
\dots,
\operatorname{vec}_C(T^d)]
\]

とする。

係数 \(c=(c_0,
\dots,c_d)^\top\) に対し

\[
S_c(T)=\sum_{k=0}^dc_kT^k
\]

と置く。この window 内で

\[
S_c(T)=\sum_iq_iF_i
\]

を証明できることと、線形方程式

\[
M_Wu=N_dc
\]

が解を持つことは同値である。

### 7.3 Window soundness

window は証明候補を制限するだけであり、数学的真理を変更してはならない。ある window で証明できないことは、\(S(T)\notin I\) を意味しない。

---

## 8. Residual Oracle

### 8.1 有限体 reduction

素数 \(p\) は、入力係数、候補係数、guard、圧縮 replay の分母を割らないとき admissible prime である。

admissible prime \(p\) に対し、\(M_W,N_d\) を \(\mathbb F_p\) 上に落としたものを

\[
M_{p,W},\quad N_{p,d}
\]

とする。

### 8.2 residual map

residual map は

\[
\rho_{p,W}:\mathbb F_p^C\to
\mathbb F_p^C/\operatorname{col}(M_{p,W})
\]

である。

実装は quotient vector を明示してもよいし、row basis handle だけで表してもよい。ただし外部契約は、任意の vector \(v\) に対して

\[
\rho_{p,W}(v)=0
\quad\Longleftrightarrow\quad
v\in\operatorname{col}(M_{p,W})
\]

を満たすことである。

### 8.3 residual-cyclic candidate search

各 target power について

\[
r_k=\rho_{p,W}(\operatorname{vec}_C(T^k))
\qquad(0\le k\le d)
\]

を計算する。

非零 \(c\in\mathbb F_p^{d+1}\) が

\[
\sum_{k=0}^dc_kr_k=0
\]

を満たすとき、

\[
S_p(T)=\sum_{k=0}^dc_kT^k\in\mathbb F_p[T]
\]

を candidate として返してよい。

これは candidate であって、証明ではない。

### 8.4 residual criterion

**定理 1（window 内 residual criterion）**  
固定した prime \(p\) と window \(W\) について、次は同値である。

1. \(M_{p,W}u=N_{p,d}c\) が解を持つ。
2. \(\rho_{p,W}(N_{p,d}c)=0\)。
3. \(\sum_{k=0}^dc_kr_k=0\)。

**証明:** residual map の定義より、\(\rho(v)=0\) は \(v\in\operatorname{col}(M)\) と同値である。\(v=Nc\) と置けばよい。

---

## 9. Candidate Oracle

candidate oracle は次の interface を満たす。

```rust
pub trait CandidateOracle {
    fn generate(&self, system: &CertifiedSystemQ, window: &CertificateWindow)
        -> Vec<TargetCandidate>;
}
```

`TargetCandidate` は次の最小情報を持つ。

```rust
pub struct TargetCandidate {
    pub support_mod_primes: Vec<UniPolynomialFp>,
    pub reconstructed: Option<UniPolynomialQ>,
    pub origin: CandidateOrigin,
    pub traces: Vec<CandidateTrace>,
}
```

候補生成器は証明書を返さない。返すのは trace だけである。

許可される origin は次に限る。

```rust
pub enum CandidateOrigin {
    DirectTargetEquation,
    NormTraceTower,
    ResidualCyclic,
    TargetCyclicKrylov,
    HiddenVariableSparseResultant,
    SliceSpecialization,
    LocalizedSchur,
    CompleteTargetElimination,
}
```

各 origin の意味は次で固定する。

### 9.1 DirectTargetEquation

入力または圧縮後の式に非零 \(S(T)\in\mathbb Q[T]\) が直接現れる場合、それを候補として返す。

### 9.2 NormTraceTower

式が代数的塔

\[
A_0\subset A_1\subset\cdots\subset A_r
\]

を明示し、\(T\) が塔の元の式で表される場合、norm

\[
\operatorname{Norm}(T-r)
\]

を候補として返してよい。

ただし norm relation も、採用には fixed proof が必要である。

### 9.3 ResidualCyclic

第 8 節の residual-cyclic search により候補を返す。

この origin は CW-ARC-DTP-Q の主候補生成器である。

### 9.4 TargetCyclicKrylov

有限 rank の target-relevant quotient handle が安価に得られる場合、

\[
1,T,T^2,
\dots
\]

の recurrence から候補を返してよい。

coverage proof がない場合でも候補として返してよいが、成功出力としては採用してはならない。

### 9.5 HiddenVariableSparseResultant

Newton support が小さい block で、\(T\) を hidden variable とする sparse resultant または eliminant template から候補を返してよい。

resultant は証明ではない。採用は exact proof による。

### 9.6 SliceSpecialization

有限体上で \(T\) 以外の変数に generic affine slice を追加し、低次元化した系から target 候補を得てよい。

slice candidate は global target relation の因子または部分観測にすぎない可能性がある。したがって、slice 間の gcd を採用の根拠にしてはならない。slice candidate は exact proof が通るまで常に候補である。

### 9.7 LocalizedSchur

localized Schur repair が exact relation を構成した場合、その relation を candidate として返す。ただし、この origin だけは同時に exact certificate を持ってよい。

### 9.8 CompleteTargetElimination

最後の target-only exact elimination により \(I:D^\infty\cap\mathbb Q[T]\) の非零元を得た場合、その多項式を返す。この origin は complete fallback であり、証明書付きでなければならない。

---

## 10. Candidate 正規化と集約

### 10.1 正規化

\(S(T)\in\mathbb Q[T]\) の候補は、採用試行前に次へ正規化する。

1. 分母を払う。
2. 整数係数の content を除く。
3. 先頭係数を正にする。
4. zero polynomial を捨てる。
5. squarefree part は root isolation 用に別に持つ。証明対象として勝手に squarefree 化してはならない。

### 10.2 候補順位

候補は次の順序で試す。

1. exact origin を持つ候補。
2. 次数が低い候補。
3. 複数 prime で再現した候補。
4. 複数 origin で再現した候補。
5. coefficient height が低い候補。
6. proof trace の active support が小さい候補。

この順位は採用条件ではない。proof attempt の順序だけを定める。

### 10.3 factor 試行

候補 \(S\) が因数分解できる場合、各因子 \(f\) を個別候補として試してよい。ただし、因子 \(f\) を採用できるのは \(f\) 自身、\(f^a\)、または \(D^ef^a\) の exact certificate が得られた場合だけである。

### 10.4 verified relation の合成

同一 certified system から得られた verified cover は gcd で絞る。

component union 由来の verified cover は lcm で合成する。

candidate 段階では gcd/lcm の結果を成功出力として扱ってはならない。合成結果も certificate を持つ場合だけ出力してよい。

---

## 11. Proof Window Learning

### 11.1 目的

proof window learning は、exact proof の線形系を小さくするための support 選択機構である。これは正しさの根拠ではない。

学習の入力は、modular candidate search で得た witness trace と、exact proof 失敗時の obstruction trace である。

### 11.2 modular witness trace

有限体上で

\[
M_{p,W}u_p=N_{p,d}c_p
\]

が解けた場合、trace は次を記録する。

\[
\operatorname{Act}_{i,p}=\{b\in B_i\mid (u_p)_{i,b}\neq0\}.
\]

複数 prime から得た active support を

\[
\operatorname{Act}_i=\bigcup_p\operatorname{Act}_{i,p}
\]

とする。

### 11.3 obstruction trace

exact proof window \(P\) で

\[
M_Pu=b
\]

が不整合である場合、左零化 witness

\[
\lambda^\top M_P=0,
\qquad
\lambda^\top b\neq0
\]

を obstruction trace として返す。

\(\lambda\) の非零成分を持つ row monomial 集合を

\[
O(\lambda)\subset\operatorname{Mon}(X,T)
\]

とする。

### 11.4 predecessor expansion

row monomial \(r\) に対し、

\[
\operatorname{Pred}_F(r)=
\{(i,b)\mid \exists \nu\in\operatorname{Supp}(F_i),\ b\nu=r\}
\]

と定義する。

obstruction trace が返った場合、次の proof window は

\[
B_i\leftarrow B_i\cup
\{b\mid (i,b)\in\operatorname{Pred}_F(r),
r\in O(\lambda)\}
\]

で拡張しなければならない。

### 11.5 exhaustive fallback と公平性

learning policy は、active support と predecessor support を優先する。ただし、resource bound が存在しない理想実行では、最終的に全 monomial support を次数順に列挙する exhaustive policy を含まなければならない。

さらに、radical proof と guarded radical proof を使う場合、探索は multiplier support だけでなく

\[
(a,e,d_B)
\]

にも公平でなければならない。ここで \(a\) は support power、\(e\) は guard power、\(d_B\) は multiplier support degree bound である。

公平性とは、任意の有限 tuple \((a,e,d_B)\) について、resource bound がない理想実行では、その tuple に対応する proof attempt がいつか必ず試されることを意味する。

実装は、例えば次の重みで探索してよい。

\[
\operatorname{weight}(a,e,d_B)=a+e+d_B.
\]

weight が小さい順に全 tuple を列挙すれば、公平性は満たされる。

したがって、ある有限 support、有限 \(a\)、有限 \(e\) の certificate が存在する場合、十分な window 拡張と公平な power 探索により fixed proof はそれを発見できる。

## 12. Fixed-S Exact Proof

### 12.1 入力

fixed proof の入力は次である。

```rust
pub struct FixedProofInput {
    pub system: CertifiedSystemQ,
    pub candidate: UniPolynomialQ,
    pub proof_window: ProofWindow,
    pub certificate_mode: CertificateMode,
}
```

`CertificateMode` は次の順序で試す。

```rust
pub enum CertificateMode {
    Ideal,
    Radical { support_power: usize },
    GuardedRadical { support_power: usize, guard_power: usize },
}
```

bounded 実行では `SolverOptions` の resource limit に従う。unbounded の理想実行では、第 11.5 節の公平性を満たす順序で `support_power` と `guard_power` を列挙しなければならない。

### 12.2 proof target

候補 \(S\) に対して proof target は

\[
H_{a,e}(T,X)=D^eS(T)^a
\]

である。

ただし

- Ideal mode では \((a,e)=(1,0)\)。
- Radical mode では \(a\ge1,e=0\)。
- GuardedRadical mode では \(a\ge1,e\ge0\)。

GuardedRadical mode の \(D\) は、`system.guard_certificates` のうち verifier が valid と認めた guard だけから構成しなければならない。

### 12.3 exact linear solve

proof target \(H\) と multiplier support \(B_i\) に対し、row set を

\[
C_H=\operatorname{Supp}(H)\cup
\bigcup_i\operatorname{Supp}(B_iF_i)
\]

とする。

\[
M_Hu=\operatorname{vec}_{C_H}(H)
\]

を \(\mathbb Q\) 上で解く。

解 \(u\) が得られたら、対応する multiplier \(q_i\) を復元し、必ず

\[
H-\sum_iq_iF_i=0
\]

を sparse polynomial identity として exact 検証する。

### 12.4 modular proof construction

実装は multiplier 探索を modular linear algebra、CRT、rational reconstruction で行ってよい。

ただし、採用条件は次だけである。

```text
- rational reconstruction 後の multiplier q_i が明示されている。
- H - Σ q_i F_i = 0 が Q 上で exact に検証される。
- GuardedRadical の場合、guard certificate が全て valid である。
```

modular solve が多数の prime で成功したこと、rank が安定したこと、reconstruction がもっともらしいことは、採用条件ではない。

### 12.5 成功条件

identity が exact に 0 になった場合だけ `TargetCertificate` を返す。

linear solve が成功しても identity check が失敗した場合は `ImplementationBug` である。

### 12.6 失敗条件

proof が失敗した場合、返す情報は次に限る。

```rust
pub enum ProofFailure {
    Inconsistent { obstruction: LeftNullObstruction },
    ResourceExceeded { trace: CostTrace },
    GuardNotCertified { guard: PolynomialQ, trace: GuardTrace },
    ReconstructionFailed { trace: ModularTrace },
}
```

`Inconsistent` は \(S\notin I\) の証明ではない。現在の proof window と現在の \((a,e)\) では証明できないことだけを意味する。

## 13. Low-degree Multiple Repair

有限体候補 \(S\) は、真の target support の因子だけを拾うことがある。この場合、\(S\) 自体の proof は失敗しても、低次数倍

\[
H(T)S(T)
\]

が membership を持つ可能性がある。

CW-ARC-DTP-Q は、次の形の repair を試してよい。

\[
P(T)=A(T)S(T)
\]

ここで \(A(T)\in\mathbb Q[T]\) は未知の低次数一変数多項式である。

ただし、\(P\) を採用するには、\(P\), \(P^a\), または \(D^eP^a\) の exact certificate が必要である。

repair により得られた support は \(P\) であり、\(S\) ではない。\(S\) 自体の certificate がない限り、\(S\) を出力してはならない。

---

## 14. Localized Schur Repair

### 14.1 目的

localized Schur repair は、fixed proof の obstruction が特定 scope に集中した場合だけ使う exact repair である。主経路ではない。

### 14.2 repair scope

obstruction trace \(\lambda\) から、row monomial と関係式 support の incidence により最小 scope

\[
\Omega\subseteq\{F_1,
\dots,F_m\}
\]

を決める。

\(\Omega\) の境界変数を

\[
Z_\Omega=\{T\}\cup\text{separators}(\Omega)
\]

とする。

### 14.3 Schur frontier space

境界 support \(A\subset\operatorname{Mon}(Z_\Omega)
\) を選び、未知境界多項式

\[
g_f(Z_\Omega)=\sum_{\alpha\in A}f_\alpha Z_\Omega^\alpha
\]

を置く。

local membership 条件は

\[
M_\Omega u+N_\Omega f=0
\]

である。

Schur repair は

\[
f\in V_\Omega
\]

を記述する constraint または basis を構成してよい。ただし、構成した relation を採用するには、元の系への exact replay certificate が必要である。

### 14.4 localized の意味

Schur repair は obstruction scope \(\Omega\) に限って行わなければならない。全体系に対する full Schur は、最後の fallback としてのみ許される。

### 14.5 Schur repair の出力

Schur repair は次のどちらかだけを返す。

1. exact certificate 付き target candidate。
2. proof window を拡張するための new support information。

証明書のない Schur relation は候補であり、成功出力ではない。

---

## 15. Complete Target Elimination Fallback

CW-ARC-DTP-Q は、最後に exact target elimination fallback を持たなければならない。

これは coordinate solving ではない。計算対象は

\[
(I:D^\infty)\cap\mathbb Q[T]
\]

である。

fallback は次のいずれかを exact に返す。

1. 非零 \(S(T)\in(I:D^\infty)\cap\mathbb Q[T]\) と guarded certificate。
2. admissible 解集合が空であることの `EmptyAdmissibleSetCertificate`。
3. \((I:D^\infty)\cap\mathbb Q[T]=\{0\}\) の algebraic certificate。
4. resource failure。

2 の場合、solver は `CertifiedEmptyAdmissibleSet` を返さなければならない。`S(T)=1` を support とする `CertifiedCandidateCover` に落としてはならない。

3 の場合でも、real admissible image の非有限性を自動で返してはならない。real 非有限性を返すには、別途 real feasibility / dimension certificate が必要である。

## 16. 主アルゴリズム

### 16.1 top-level pseudocode

```text
solve_target_cw_arc(problem):

    P0 = Validate(problem)
    P1 = Canonicalize(P0)
    P  = CertifiedCompress(P1)

    early_empty = TryCheapEmptyAdmissibleSetCertificate(P)
    if early_empty.valid:
        return CertifiedEmptyAdmissibleSet(early_empty.certificate)

    D  = BuildTargetDependencyDAG(P)
    Ws = PlanCertificateWindows(P, D)

    Verified = []
    collected_obstructions = []

    for W in Ws:

        Cands = []

        Cands += DirectTargetEquation(P, W)
        Cands += NormTraceTower(P, W)
        Cands += ResidualCyclicModP(P, W)
        Cands += TargetCyclicKrylov(P, W)
        Cands += HiddenVariableSparseResultant(P, W)
        Cands += SliceSpecialization(P, W)

        for S in NormalizeAndRank(Cands):

            for Fctr in FactorSchedule(S):

                PW = LearnInitialProofWindow(W, Fctr.traces)

                for mode in FairCertificateModeSchedule(options):

                    result = ProveFixedTarget(P, Fctr, PW, mode)
                    if result.valid:
                        Verified += result.certificate
                        cover = RefineAndFinalize(Verified, SameIdealGcd)
                        return MaybeClassifyExactTargetImage(problem, cover, options)

                    while result is Inconsistent and CanExpand(PW):
                        collected_obstructions += result.obstruction
                        PW = ExpandByObstructionPredecessors(PW, result.obstruction)
                        result = ProveFixedTarget(P, Fctr, PW, mode)
                        if result.valid:
                            Verified += result.certificate
                            cover = RefineAndFinalize(Verified, SameIdealGcd)
                            return MaybeClassifyExactTargetImage(problem, cover, options)

                repaired = LowDegreeMultipleRepair(P, Fctr, PW)
                if repaired.valid:
                    Verified += repaired.certificate
                    cover = RefineAndFinalize(Verified, SameIdealGcd)
                    return MaybeClassifyExactTargetImage(problem, cover, options)

            schur = LocalizedSchurRepair(P, W, collected_obstructions)
            if schur.valid:
                Verified += schur.certificate
                cover = RefineAndFinalize(Verified, SameIdealGcd)
                return MaybeClassifyExactTargetImage(problem, cover, options)

    final = CompleteTargetEliminationFallback(P)

    if final.has_support:
        cover = FinalizeCertifiedCover(final.certificate)
        return MaybeClassifyExactTargetImage(problem, cover, options)

    if final.certifies_empty_admissible_set:
        return CertifiedEmptyAdmissibleSet(final.certificate)

    if final.certifies_no_algebraic_target_constraint:
        return CertifiedNoNonzeroTargetEliminant(final.certificate)

    return NoVerifiedTargetCertificate(final.failure_trace)
```

### 16.2 MaybeClassifyExactTargetImage

`MaybeClassifyExactTargetImage` は、`SolverOptions` により exact image が要求される場合だけ root fiber classification を試みる。

```text
MaybeClassifyExactTargetImage(problem, cover, options):

    if options.exact_image_mode == CoverOnly:
        return CertifiedCandidateCover(cover)

    image = ClassifyRealFibers(problem, cover)

    if image.valid_and_complete:
        return CertifiedExactTargetImage(image.result)

    if options.exact_image_mode == RequireExactImage:
        return NoVerifiedTargetCertificate(image.failure_trace)

    return CertifiedCandidateCover(cover.with_trace(image.partial_trace))
```

`AllowPartialExactImage` のような option を設ける場合でも、部分分類は trace であり、`CertifiedExactTargetImage` ではない。

### 16.3 FinalizeCertifiedCover

成功 certificate から support polynomial \(S\) を取り出し、

\[
S_{sq}=\operatorname{squarefree\_part}(S)
\]

を root isolation に渡す。

出力は

```rust
pub struct CertifiedCandidateCover {
    pub support: UniPolynomialQ,
    pub squarefree_support: UniPolynomialQ,
    pub real_roots: Vec<AlgebraicRealRoot>,
    pub certificate: TargetCertificate,
}
```

である。

## 17. Target Dependency DAG と window planning

### 17.1 DAG の目的

Target Dependency DAG は、局所 projection message を合成するためではなく、certificate window の scope を決めるために使う。

DAG は次の algebraic footprint だけを見る。

```text
- relation-variable incidence
- target からの graph distance
- separator size
- monomial support
- degree
- affine eliminability
- explicit tower detectability
- quotient-rank estimate
```

幾何名、変数名、期待答えは使ってはならない。

### 17.2 window planner

planner は次を満たす window 列

\[
W_1,W_2,
\dots
\]

を作る。

1. 各 \(W_j\) は finite, row-closed である。
2. \(W_j\) は target dependency cone を優先する。
3. \(W_j\) は小さい separator support を優先する。
4. 無制限実行では、window 列は最終的に全 monomial multiplier support を次数順に網羅する。

planner の estimate は正しさに影響してはならない。

---

## 18. 正しさ定理

### 18.1 Ideal membership soundness

**定理 2**  
\(S(T)=\sum_iq_iF_i\) が \(R\) 上の恒等式として成り立つなら、任意の解 \((x,t)\in V(I)\) は \(S(t)=0\) を満たす。

**証明:** 解では全ての \(F_i(x,t)=0\) である。したがって \(S(t)=\sum_iq_i(x,t)F_i(x,t)=0\)。

### 18.2 Radical soundness

**定理 3**  
\(S(T)^a=\sum_iq_iF_i\) が \(a\ge1\) について成り立つなら、任意の解で \(S(t)=0\) である。

**証明:** 解では \(S(t)^a=0\)。体上なので \(S(t)=0\)。

### 18.3 Guarded radical soundness

**定理 4**  
\(D^eS(T)^a=\sum_iq_iF_i\) が成り立ち、admissible 解で \(D\neq0\) なら、任意の admissible 解で \(S(t)=0\) である。

**証明:** admissible 解では右辺が 0 であり、\(D^e\neq0\)。よって \(S(t)^a=0\)、したがって \(S(t)=0\)。

### 18.4 Guard certificate soundness

**定理 5**  
`GuardCertificate` が valid であり、その guard product が \(D\) と exact に一致するなら、任意の admissible 解で \(D\neq0\) である。

**証明:** 各 factor は certificate により admissible 解上で非ゼロである。体上では非ゼロ元の積は非ゼロであるため、\(D\neq0\) である。

### 18.5 Empty admissible set soundness

**定理 6**  
`EmptyAdmissibleSetCertificate` が valid なら、admissible 解集合は空である。

**証明:** `AlgebraicInfeasibility` では解に代入すると \(1=0\) となり矛盾する。`GuardedAlgebraicInfeasibility` では、admissible 解に代入すると右辺は 0 だが、guard certificate により \(D^e\neq0\) であり矛盾する。`RealInfeasibility` は実閉体上の infeasibility certificate が直接これを示す。

### 18.6 Candidate oracle non-soundness

**定理 7**  
有限体 residual、Krylov recurrence、slice specialization、sparse resultant が返す候補は、それだけでは \(S(T)\in I\) を含意しない。

**仕様上の帰結:** これらの候補は exact certificate が付くまで採用してはならない。

### 18.7 Same-ideal gcd refinement

**定理 8**  
同一 admissible 解集合に対して verified cover \(S_1,
\dots,S_k\) が成り立つなら、

\[
G=\gcd(S_1,
\dots,S_k)
\]

も verified cover である。

**証明:** 任意の target 値 \(t\) は全ての \(S_i\) の根である。したがって \(t\) はそれらの共通根集合、すなわち \(G\) の根に含まれる。

### 18.8 Component-union lcm rule

**定理 9**  
component \(J_1,
\dots,J_k\) の target cover が \(S_1,
\dots,S_k\) であり、全体が component union semantics を持つなら、

\[
L=\operatorname{lcm}(S_1,
\dots,S_k)
\]

は全体の cover である。

**証明:** 任意の解はどれか 1 つの component に属する。その component の support の根であるため、lcm の根でもある。

### 18.9 Window learning soundness

**定理 10**  
proof window learning がどの support を選んでも、採用が exact verifier によってのみ行われる限り、出力の soundness は変わらない。

**証明:** learning は proof attempt の探索空間を変えるだけであり、成功判定は第 6 節の証明書言語で行われるため。

### 18.10 Fair proof search completeness for finite certificates

**定理 11**  
resource bound がなく、ある候補 \(S\) について有限 support、有限 \(a\)、有限 \(e\) の certificate が存在し、かつ candidate schedule がその \(S\) をいつか生成するなら、公平な proof search はその certificate をいつか発見する。

**証明:** 第 11.5 節の公平性により、certificate に必要な \((a,e,d_B)\) はいつか試される。exhaustive support policy により必要な monomial support もいつか window に含まれる。その時点で exact linear solve は certificate の multiplier を含む解空間を探索するため、identity check により採用される。

### 18.11 Complete fallback completeness

**定理 12**  
resource bound がなく、exact elimination が完了するなら、CompleteTargetEliminationFallback は \((I:D^\infty)\cap\mathbb Q[T]\) が非零かどうかを判定できる。

**証明:** 多項式環上の elimination theory により、Groebner basis などの exact elimination は elimination ideal の生成系を計算できる。saturation は補助変数または標準的な ideal quotient 計算で表現できる。

### 18.12 Exact target image soundness

**定理 13**  
`ExactTargetImageCertificate` が valid であり、`squarefree_support` の全実根が `Nonempty` または `Empty` に分類されているなら、`Nonempty` に分類された根だけが real admissible target image である。

**証明:** target cover certificate により、任意の real admissible target 値は support の実根である。各実根について fiber の非空性または空性が exact に証明されているため、非空と証明された根だけが実現可能な target 値である。

## 19. 計算量モデル

記号を次で固定する。

| 記号 | 意味 |
|---|---|
| \(D_{full}\) | full coordinate quotient rank または座標解数の尺度 |
| \(\delta\) | target support degree |
| \(|W|\) | certificate window の行列サイズ |
| \(C_{res}(W)\) | residual oracle 1 回の構築・更新コスト |
| \(C_{proof}(S,W)\) | fixed proof の exact linear solve と identity check のコスト |
| \(C_{schur}(\Omega)\) | localized Schur repair のコスト |
| \(\tau\) | separator 幅 |

理想的な成功時の支配項は

\[
\widetilde O(\delta C_{res}(W))
+ C_{proof}(S,W_S)
\]

である。

Schur 主経路の場合は frontier 空間全体の構成が必要になり、概ね

\[
\sum_\Omega C_{schur}(\Omega)
\]

に支配される。

CW-ARC-DTP-Q の狙いは、frontier 空間全体を作らず、まず \(S(T)\) の候補だけを拾い、その \(S(T)\) だけを fixed proof することである。

この設計が特に効くのは次の場合である。

```text
- 変数は多いが incidence graph が疎である。
- target dependency cone が小さい。
- full quotient は大きい、または positive-dimensional だが target image は有限である。
- target support degree δ が小さい。
- short membership certificate が存在する。
- modular witness の active support が小さい。
- separator monomial の実使用量が少ない。
```

苦しい場合は次である。

```text
- dense system である。
- target eliminant degree が大きい。
- short certificate が存在しない。
- guard saturation が高次数を要求する。
- proof window expansion がすぐ全体化する。
```

この場合でも、solver は unsound な成功を返してはならない。

---

## 20. API contract

最小 API は次である。

```rust
pub fn solve_target(problem: TargetProblemQ, options: SolverOptions)
    -> TargetSolveResult;
```

`SolverOptions` は resource と exact image の要求度だけを指定する。algorithmic branch を幾何名で指定する option を持ってはならない。

```rust
pub struct SolverOptions {
    pub resource_limits: ResourceLimits,
    pub exact_image_mode: ExactImageMode,
}

pub enum ExactImageMode {
    CoverOnly,
    TryExactImage,
    RequireExactImage,
}
```

```rust
pub struct TargetSolveResult {
    pub status: SolverStatus,
    pub cover: Option<CertifiedCandidateCover>,
    pub exact_image: Option<CertifiedExactTargetImage>,
    pub certificate: Option<SolverCertificate>,
    pub trace: SolverTrace,
}
```

status は次に限る。

```rust
pub enum SolverStatus {
    CertifiedCandidateCover,
    CertifiedExactTargetImage,
    CertifiedEmptyAdmissibleSet,
    CertifiedNoNonzeroTargetEliminant,
    NoVerifiedTargetCertificate,
    FiniteResourceFailure,
    CertificateDesignGap,
    InvalidInput,
    ImplementationBug,
}
```

`CertifiedEmptyAdmissibleSet` は、入力条件を満たす admissible 解が存在しないことを意味する。これは target cover とは別 status である。

`CertifiedNoNonzeroTargetEliminant` は algebraic statement であり、real target image の非有限性を意味しない。real 非有限性を返す status は、別途 real certificate を持つ場合に限って追加してよい。

## 21. Verifier contract

verifier は solver 本体から独立して再実行できなければならない。

```rust
pub fn verify_certificate(problem: TargetProblemQ, cert: SolverCertificate)
    -> VerificationResult;
```

verifier は次だけを信頼する。

```text
- 入力多項式
- 入力に実際に含まれる semantic guard record
- certificate 内の multiplier
- replay certificate
- guard certificate の exact identity
- real infeasibility / real fiber certificate の検証可能な payload
- 有理数演算
```

verifier は次を信頼してはならない。

```text
- modular rank trace
- random seed
- candidate score
- planner estimate
- floating point result
- external CAS output without replay
- guard_product だけで与えられた非ゼロ主張
- partial real fiber classification
```

`TargetCertificate` だけを検証する helper を持ってよいが、top-level result の検証では必ず `SolverCertificate` を検証しなければならない。

## 22. Exact real root and exact target image stage

candidate cover が得られたら、

\[
S_{sq}=\operatorname{squarefree\_part}(S)
\]

を作り、exact real root isolation を行う。

root isolation は rational isolating interval または同等の exact algebraic root record を返さなければならない。floating approximation だけを返してはならない。

`CertifiedCandidateCover` では、各実根は「候補値」であり、実現可能性までは主張しない。

### 22.1 root record

各 root は、少なくとも次を持つ。

```rust
pub struct AlgebraicRealRoot {
    pub polynomial: UniPolynomialQ,
    pub isolating_interval: RationalInterval,
    pub index: usize,
}
```

`polynomial` は squarefree でなければならず、`isolating_interval` はちょうど 1 つの実根だけを含まなければならない。

### 22.2 real fiber classification

各 root \(\alpha\) について、次を exact に判定する。

\[
\exists X\in\mathbb R^n
\quad
F(X,\alpha)=0
\quad\land\quad
\text{semantic guards are satisfied}.
\]

判定結果は次のどちらかでなければならない。

```text
- Nonempty: 実 admissible fiber が非空であることの certificate。
- Empty: 実 admissible fiber が空であることの certificate。
```

数値的な近似解、sampling、floating point feasibility は certificate ではない。

### 22.3 exact image success condition

`CertifiedExactTargetImage` を返すには、`squarefree_support` の全ての実根が `Nonempty` または `Empty` に分類されていなければならない。

一つでも未分類の根がある場合、solver は次のいずれかにしなければならない。

```text
- CoverOnly / TryExactImage では CertifiedCandidateCover を返す。
- RequireExactImage では NoVerifiedTargetCertificate または FiniteResourceFailure を返す。
```

未分類の根を黙って捨ててはならない。

## 23. 仕様全体の不変条件

CW-ARC-DTP-Q の全実行は次を満たさなければならない。

```text
I1. 入力は任意の well-formed Q-polynomial target system として受ける。
I2. 幾何名、問題名、期待答えで分岐しない。
I3. production path は全座標解を作らない。
I4. production path は full coordinate RUR を作らない。
I5. 候補生成と証明を分離する。
I6. 成功出力は exact certificate を持つ。
I7. guard を使う場合は GuardCertificate を持つ。
I8. finite-field / slice / Krylov / resultant trace は採用条件にならない。
I9. same ideal の verified cover は gcd で絞る。
I10. component union だけ lcm を使う。
I11. nonfinite / no eliminant は complete exact certificate なしに返さない。
I12. verifier は solver trace なしで certificate を再検証できる。
I13. empty admissible set は cover ではなく CertifiedEmptyAdmissibleSet として返す。
I14. Radical / GuardedRadical の unbounded 探索は support degree, support power, guard power に公平である。
I15. modular proof construction を使っても、採用は Q 上の exact identity check に限る。
I16. CertifiedExactTargetImage は全実根の real fiber classification が完了した場合だけ返す。
I17. partial real fiber classification は trace であり、success certificate ではない。
```

---

## 24. 参照設計原則

本仕様は、アルゴリズム上の正しさとは別に、次の設計原則を構造面の基準として採用する。

1. **仕様品質:** 仕様は正確、一意、完全、一貫、検証可能でなければならない。
2. **情報隠蔽:** 変わりやすい設計判断、例えば residual oracle の内部表現や matrix backend は外部 contract から隠す。
3. **最小公開面:** API は solver の本質 object、つまり problem、candidate cover、certificate、trace だけを公開する。
4. **fail-closed:** 証明できない場合は成功を返さない。
5. **explicitness:** 候補、証明、guard、fallback、status の意味を型と certificate で明示する。

---

## 25. 最終仕様文

CW-ARC-DTP-Q の production path は、次の規則で完全に定義される。

```text
The solver is target-certificate-first.
It searches for target-only candidate polynomials by modular residual probes,
Krylov probes, sparse resultant probes, norm/trace probes, and slice probes.
No candidate is trusted.
A candidate is accepted only if the original rational system, or a certified
replay-equivalent system, admits an exact certificate proving S(T), S(T)^a,
or D^e S(T)^a in the input ideal.
Every guard used in D must have a separately verifiable GuardCertificate.
Schur computation is not the primary object; it is used only for localized
proof repair or final exact target elimination.
```

この仕様の標準成功出力は、「値そのもの」ではなく、まず「値を覆う exact support polynomial」である。実数候補値はその exact root isolation として得る。余分な根を除く処理は後段の exact real fiber classification であり、candidate cover の soundness とは分離する。

admissible 解集合が空である場合は、任意の support polynomial を返すのではなく、`CertifiedEmptyAdmissibleSet` を返す。全実根の real fiber classification が完了した場合だけ、`CertifiedExactTargetImage` を返す。

## 26. 参考文献

この節はアルゴリズムの正しさを外部文献に依存させるものではない。設計上の位置づけと実装時に参照すべき代表文献を明示するために置く。

1. IEEE Std 830-1998, *IEEE Recommended Practice for Software Requirements Specifications*. 仕様が correct, unambiguous, complete, consistent, verifiable であるべきことを示す古典的基準。
2. D. L. Parnas, *On the Criteria To Be Used in Decomposing Systems into Modules*, Communications of the ACM, 1972. 情報隠蔽と module boundary の設計原則。
3. Rust API Guidelines. Rust ecosystem における明示的で一貫した API 設計指針。
4. J.-C. Faugère, *A new efficient algorithm for computing Gröbner bases (F4)*, Journal of Pure and Applied Algebra, 1999. 疎線形代数による batch reduction の基礎。
5. D. H. Wiedemann, *Solving sparse linear equations over finite fields*, IEEE Transactions on Information Theory, 1986. 有限体上の疎線形 recurrence / Krylov 型手法の基礎。
6. J.-C. Faugère and C. Mou, *Sparse FGLM algorithms*, arXiv:1304.1238. multiplication matrix の疎性を利用した target-only recurrence 設計の参考。
7. D. Cifuentes and P. A. Parrilo, *Exploiting chordal structure in polynomial ideals*, arXiv:1411.1745. chordal / low-treewidth 構造を代数計算に使うための参考。
8. C. D’Andrea, G. Jeronimo, and M. Sombra, *The Canny–Emiris Conjecture for the Sparse Resultant*, arXiv:2004.14622. sparse resultant を候補生成器として扱う際の参考。
9. J. Berthomieu, C. Eder, M. Safey El Din et al., *msolve: A Library for Solving Polynomial Systems*, arXiv:2104.03572. full zero-dimensional solver との比較基準。
