# GeoSolver Core Algorithm Specification v4.0
# R-GDTPK-Q / ACCTP-Q: Algebraic-Cost Compressed Direct Target Projection over Rational Polynomial Systems

作成日: 2026-07-04  
文書種別: **アルゴリズム設計固定用の自己完結仕様書**  
対象: `geosolver-core` の solver core  
主目的: 幾何問題から得られる大規模な有理数係数多項式系から、全座標解を先に求めず、target 値候補を高速かつ証明付きで導出する。  
非目的: benchmark 設計、テスト設計、実験計画、自然言語・図認識、幾何DSL lowering の詳細。

---

## 0. この文書の読み方

この文書は、実装者がこの会話履歴や過去の計画書を知らなくても `geosolver-core` を実装できるように、数学的アルゴリズム、データ構造、処理順序、folder 構成、各 file に置く関数、各関数の入力・処理・出力を固定する仕様書である。

この文書では、次の名前を使う。

```text
R-GDTPK-Q
    Rational Generic Direct Target Projection Kernel over Q-polynomial systems

ACCTP-Q
    Algebraic-Cost Compressed Target Projection over Q-polynomial systems
```

本仕様では、`R-GDTPK-Q` と `ACCTP-Q` は同じ solver core を指す。前者は「有理数係数多項式系上の汎用 target-direct kernel」という性質を表し、後者は「代数コスト圧縮」という研究上の強みを表す。

---

## 1. 研究上の強み

### 1.1 主張する強み

この研究の強みは、次である。

```text
幾何由来の大規模な有理数係数多項式系 F ⊂ Q[x1,...,xn,T] から、
全座標解や全座標RURを構成せず、
target T と separator 変数に必要な代数関係だけを局所的に射影・合成する。

これにより、全体消去の代数コストを、全変数数 n ではなく、
局所 block 幅、separator 幅、local quotient/action rank、sparse template size、
最終 support degree に依存させる。
```

この仕様が目指す高速化は、単に「target 値の個数が座標解数より少ないから速い」というものではない。target 値数と座標解数が同程度の場合でも、次が小さければ高速化の余地がある。

```text
- target に到達するまでの局所 block 幅
- separator 変数の個数と次数
- 各局所 block の quotient rank
- 各局所 block の sparse resultant / Macaulay template size
- multiplication-by-target operator の疎性
- 証明書の検証コスト
```

### 1.2 強みの正確な言い換え

悪い説明:

```text
target-direct なので、常に homotopy や Groebner より速い。
```

採用する説明:

```text
R-GDTPK-Q は、full coordinate solution object を作らず、
target/separator projection message を TargetProjectionDAG 上で合成する。
幾何由来の多項式系が持ちやすい低次数・疎性・局所依存・小 separator を、
幾何名ではなく代数的 footprint として利用することで、
全体消去に必要な巨大な代数 object を局所 object の列へ分解する。
```

### 1.3 汎用性に関する絶対条件

この solver は、特定の幾何形式だけを support する solver ではない。

禁止される設計:

```text
- circle solver
- triangle solver
- tangent solver
- metric-only solver
- distance-only solver
- area-only solver
- target-univariate-only solver
- affine-only solver
- bivariate-only solver
- no-branch-only solver
- no-slack-only solver
- ある形の式だけを完成扱いする solver
```

正しい設計:

```text
入力は任意の well-formed な有理数係数多項式系。
すべての入力は generic target projection pipeline に入る。
特定形でないことを理由に Unsupported を返してはならない。
計算できない場合は、どの代数的パラメータが障害になったかを
FiniteResourceFailure / AlgorithmicHardCase / CertificateDesignGap として返す。
```

---

## 2. 数学的問題設定

### 2.1 入力

入力は次である。

```text
F = {f1,...,fm} ⊂ Q[x1,...,xn,T]
T = target variable
```

各 `fi` は有理数係数 sparse polynomial である。変数は、座標変数、補助変数、slack 変数、selector 変数、構成変数などを含んでよい。ただし solver core は、変数 role を計算の分岐条件にしてはならない。

`F` が生成する ideal を次とする。

```text
I = <F> ⊂ Q[x1,...,xn,T]
```

候補値を得るには target elimination ideal

```text
I_T = I ∩ Q[T]
```

の非零元を求めればよい。非零元 `S(T) ∈ I_T` が得られれば、任意の複素解 `x,T` は `S(T)=0` を満たす。したがって実数幾何としての真の target 値も `S` の実根に含まれる。

### 2.2 出力

主出力は次である。

```text
S(T) ∈ Q[T]
```

ただし `S(T)` は次を満たす。

```text
S(T) != 0
true target values ⊆ roots(S)
```

`S` は squarefree 化して root isolation へ渡す。

```text
S_sq(T) = squarefree_part(S(T))
RealRootIsolation(S_sq)
```

各実根は rational isolating interval または algebraic-root record として出力する。

### 2.3 Candidate cover と exact target image

本仕様では、次の二段階を厳密に分ける。

```text
CertifiedCandidateCover:
    S(T) を返す。
    真の target 値は roots(S) に含まれる。
    spurious roots は含まれてよい。

CertifiedExactTargetImage:
    roots(S) の各実根について real fiber / guard / slack semantics を判定し、
    実際に実現可能な target 値だけを返す。
```

この仕様の必須 core は `CertifiedCandidateCover` である。`CertifiedExactTargetImage` は同じ architecture の後段として設計に含める。

### 2.4 不等式・非ゼロ条件

幾何問題では不等式や非ゼロ条件が出る。core input は等式系として受けるので、これらは必要に応じて補助変数で等式化される。

```text
A >= 0   ->   A - s^2 = 0
A > 0    ->   A*s^2 - 1 = 0
A != 0   ->   A*s - 1 = 0
```

ただし、これは実数 semantics を表すための encoding であり、複素数上の elimination だけで最終的な実数条件を判定してはならない。等式化された guard は provenance を持ち、exact image mode の RealFiberClassifier が解釈する。

---

## 3. 非交渉の設計原則

### 3.1 禁止事項

次は禁止である。

```text
1. full coordinate solution list を production path で作る。
2. full coordinate RUR を production path で作る。
3. 全座標 lex parametrization を作ってから T を読む。
4. generic QE/CAD を隠れ fallback として使う。
5. 外部CASの答えをそのまま certified production proof とする。
6. ある kernel が失敗した後、planner に記録されていない hidden fallback を呼ぶ。
7. 幾何名、問題名、fixture 名、期待答え、公式解に基づいて分岐する。
8. 特定の式形だけを support として、それ以外を Unsupported とする。
```

### 3.2 許可される重い代数処理

次は許可される。

```text
1. local block 内での target/separator elimination
2. exact modular F4/F5-like sparse linear algebra
3. target/separator variables だけを export する resultant / eliminant
4. multiplication-by-target operator
5. target-relevant quotient/action handle
6. sparse resultant template
7. regular-chain projection
8. norm/trace computation
9. specialization-interpolation
```

許可条件は次である。

```text
- deterministic planner が実行前に選ぶ。
- export は target/separator relation のみ。
- coordinate roots を返さない。
- full coordinate RUR を返さない。
- exact Q verification を持つ。
- cost trace と certificate を出す。
```

### 3.3 失敗の扱い

well-formed な Q-polynomial input に対して、通常の `Unsupported` は返さない。

許可される失敗 status:

```text
FiniteResourceFailure:
    指定された resource bound 内で exact target-direct computation が完了しない。

AlgorithmicHardCase:
    現在の target-direct algorithm では support relation を構成できない。
    ただし、最小障害 block、matrix size、rank estimate、degree bound などを返す。

CertificateDesignGap:
    候補 object は構成できたが、現行 certificate language で検証できない。

ImplementationBug:
    仕様上成立すべき invariant が破れた。

InvalidInput:
    Q-polynomial system として well-formed でない。
```

---

## 4. 全体アルゴリズム

### 4.1 Top-level pipeline

```text
solve_target(problem, options):
    0. ValidateInput
    1. CanonicalizeSystem
    2. PreKernelAlgebraicCompression
    3. BuildRelationVariableHypergraph
    4. BuildTargetInfluenceGraph
    5. BuildWeightedProjectionGraph
    6. BuildTargetProjectionDAG
    7. PlanProjectionMessages
    8. ExecuteLocalProjectionKernels
    9. ComposeProjectionMessages
   10. BuildGlobalSupportPolynomial
   11. VerifyGlobalSupport
   12. SquarefreeSupport
   13. ExactRealRootIsolation
   14. DecodeTargetCandidates
   15. OptionalRealFiberClassification
   16. FinalizeResultAndCertificate
```

### 4.2 重要な invariant

各 step は次を守る。

```text
I1. 入力は Q-polynomial system として扱う。
I2. 幾何名で分岐しない。
I3. production path は coordinate solution list を作らない。
I4. 各 block は target/separator variables だけを export する。
I5. 各 exported relation は exact Q certificate を持つ。
I6. final S(T) は candidate cover として exact に検証される。
I7. exact image mode では real fiber / slack / guard semantics を検証する。
I8. 失敗時は、障害となった代数コストを trace に出す。
I9. planner は deterministic である。
I10. hidden fallback はない。
```

---

## 5. 主要データ型

この節の型は Rust 風疑似コードで書く。実装言語が Rust でない場合も、同じ field と invariant を持つ構造にする。

### 5.1 ID 型

```rust
pub struct VariableId(pub u32);
pub struct RelationId(pub u32);
pub struct BlockId(pub u32);
pub struct PackageId(pub u32);
pub struct KernelPlanId(pub u32);
pub struct Hash(pub [u8; 32]);
```

全 ID は stable であり、canonicalization 後に再現可能でなければならない。

### 5.2 有理数

```rust
pub struct RationalQ {
    pub num: BigInt,
    pub den: BigInt, // always positive
}
```

Invariant:

```text
- gcd(num, den) = 1
- den > 0
- zero is represented as 0/1
```

### 5.3 Monomial と polynomial

```rust
pub struct Monomial {
    pub exponents: Vec<(VariableId, u32)>, // sorted by VariableId, no zero exponent
}

pub struct TermQ {
    pub coeff: RationalQ,
    pub monomial: Monomial,
}

pub struct SparsePolynomialQ {
    pub terms: Vec<TermQ>, // sorted by monomial order, no zero coeff, no duplicate monomial
    pub hash: Hash,
}

pub struct UniPolynomialQ {
    pub variable: VariableId,
    pub coeffs_low_to_high: Vec<RationalQ>,
    pub hash: Hash,
}
```

### 5.4 Problem input

```rust
pub struct RationalTargetProblem {
    pub variables: Vec<VariableId>,
    pub target: VariableId,
    pub equations: Vec<SparsePolynomialQ>,
    pub semantic_encodings: Vec<RealConstraintEncoding>,
    pub variable_roles: Vec<VariableRoleRecord>,
    pub input_hash: Hash,
}

pub struct RealConstraintEncoding {
    pub original_kind: RealConstraintKind,
    pub encoded_relation_ids: Vec<RelationId>,
    pub slack_variables: Vec<VariableId>,
    pub semantic_hash: Hash,
}
```

`variable_roles` は provenance 用であり、algorithmic dispatch に使ってはならない。

### 5.5 Canonical system

```rust
pub struct CanonicalSystemQ {
    pub variables: Vec<VariableId>,
    pub target: VariableId,
    pub relations: Vec<CanonicalRelationQ>,
    pub relation_order: Vec<RelationId>,
    pub variable_order: VariableOrder,
    pub semantic_encodings: Vec<RealConstraintEncoding>,
    pub canonical_hash: Hash,
}

pub struct CanonicalRelationQ {
    pub id: RelationId,
    pub polynomial: SparsePolynomialQ,
    pub source: RelationSource,
    pub hash: Hash,
}
```

### 5.6 Projection message

従来の単一 `TargetRelationPackage` は弱い。separator が複数ある場合、projection ideal は一般に principal ではない。したがって、本仕様では projection message を複数 relation を含められる形にする。

```rust
pub struct ProjectionMessage {
    pub package_id: PackageId,
    pub block_id: BlockId,
    pub kernel_kind: KernelKind,
    pub source_relation_ids: Vec<RelationId>,
    pub eliminated_variables: Vec<VariableId>,
    pub exported_variables: Vec<VariableId>, // subset of {T}+separators
    pub relation_generators: Vec<SparsePolynomialQ>,
    pub representation: MessageRepresentation,
    pub projection_strength: ProjectionStrength,
    pub certificate: KernelCertificate,
    pub compression_trace: CompressionTrace,
    pub cost_trace: ProjectionCostTrace,
    pub package_hash: Hash,
}

pub enum MessageRepresentation {
    GeneratorSet,
    PrincipalSupport,
    TriangularChain,
    QuotientAction,
    NormTraceTower,
    SparseResultantMatrix,
    SpecializationInterpolation,
}

pub enum ProjectionStrength {
    CandidateCoverWeak,
    CandidateCoverStrong,
    RadicalProjectionApprox,
    ExactProjectionIdeal,
    ExactRealFiberAware,
}
```

Invariant:

```text
- exported_variables に eliminated local variables は含まれない。
- relation_generators は Q[exported_variables] に属する。
- package は coordinate roots を含まない。
- package は full coordinate RUR を含まない。
- certificate は modular-only ではない。最終的に Q 上で検証できる。
```

### 5.7 Result

```rust
pub struct TargetSolveResult {
    pub status: SolverStatus,
    pub target: VariableId,
    pub support_polynomial: Option<UniPolynomialQ>,
    pub squarefree_support_polynomial: Option<UniPolynomialQ>,
    pub root_isolation: Vec<RealRootRecord>,
    pub decoded_candidates: Vec<TargetCandidate>,
    pub projection_messages: Vec<ProjectionMessage>,
    pub certificate: Option<CoreRunCertificate>,
    pub diagnostics: Vec<DiagnosticRecord>,
    pub cost_trace: GlobalCostTrace,
}

pub enum SolverStatus {
    CertifiedCandidateCover,
    CertifiedExactTargetImage,
    CertifiedEmptyRealTargetImage,
    CertifiedNonFiniteTargetImage,
    FiniteResourceFailure,
    AlgorithmicHardCase,
    CertificateDesignGap,
    ImplementationBug,
    InvalidInput,
}
```

---

## 6. Folder 構成

実装 folder は次の構成に固定する。

```text
geosolver-core/
  Cargo.toml
  README.md
  src/
    lib.rs
    api.rs

    types/
      mod.rs
      ids.rs
      rational.rs
      monomial.rs
      polynomial.rs
      univariate.rs
      matrix.rs
      interval.rs
      hash.rs

    problem/
      mod.rs
      input.rs
      semantic.rs
      validate.rs
      canonicalize.rs
      context.rs

    algebra/
      mod.rs
      monomial_order.rs
      polynomial_ops.rs
      modular.rs
      crt.rs
      rational_reconstruction.rs
      sparse_matrix.rs
      dense_matrix.rs
      linear_solve.rs
      normal_form.rs
      groebner.rs
      f4.rs
      elimination.rs
      resultant.rs
      interpolation.rs
      quotient.rs
      krylov.rs
      regular_chain.rs
      norm_trace.rs
      real_root.rs
      sign.rs

    preprocess/
      mod.rs
      compression.rs
      linear_affine.rs
      definitional.rs
      binomial.rs
      saturation.rs
      independent.rs

    graph/
      mod.rs
      hypergraph.rs
      influence.rs
      weighted_primal.rs
      separators.rs
      tree_decomposition.rs
      projection_dag.rs
      metrics.rs

    planner/
      mod.rs
      cost_model.rs
      probes.rs
      admission.rs
      kernel_plan.rs
      ladder.rs
      planner.rs

    kernels/
      mod.rs
      traits.rs
      target_univariate.rs
      linear_affine.rs
      target_relation_search.rs
      sparse_resultant.rs
      action_krylov.rs
      universal_elimination.rs
      regular_chain_projection.rs
      norm_trace_projection.rs
      specialization_interpolation.rs

    compose/
      mod.rs
      message.rs
      compose.rs
      separator_elimination.rs
      final_support.rs

    verify/
      mod.rs
      certificates.rs
      verify_message.rs
      verify_support.rs
      replay.rs
      run_certificate.rs

    roots/
      mod.rs
      squarefree.rs
      isolate.rs
      decode.rs
      algebraic_number.rs

    fiber/
      mod.rs
      exact_image.rs
      hermite.rs
      thom.rs
      slack_semantics.rs

    result/
      mod.rs
      status.rs
      diagnostics.rs
      cost_trace.rs
      output.rs

    solver/
      mod.rs
      options.rs
      pipeline.rs
      orchestrator.rs
```

`tests/` や benchmark folder はこの仕様書では定義しない。

---

## 7. File 別仕様: public API

### 7.1 `src/lib.rs`

目的: crate の public module を公開する。

必須内容:

```rust
pub mod api;
pub mod types;
pub mod problem;
pub mod algebra;
pub mod preprocess;
pub mod graph;
pub mod planner;
pub mod kernels;
pub mod compose;
pub mod verify;
pub mod roots;
pub mod fiber;
pub mod result;
pub mod solver;
```

禁止:

```text
- lib.rs に solver logic を直接書かない。
- geometry-specific helper を公開しない。
```

### 7.2 `src/api.rs`

目的: 外部から呼ぶ単一 API を定義する。

関数:

```rust
pub fn solve_target(
    problem: RationalTargetProblem,
    options: SolverOptions,
) -> TargetSolveResult
```

入力:

```text
problem: 有理数係数多項式系と target
options: resource bound、exact image mode、certificate level など
```

処理:

```text
1. SolverContext を初期化する。
2. solver::orchestrator::solve_with_context を呼ぶ。
3. panic ではなく SolverStatus 付き result を返す。
```

出力:

```text
TargetSolveResult
```

疑似コード:

```rust
pub fn solve_target(problem, options) -> TargetSolveResult {
    let ctx = SolverContext::new(options);
    match solver::orchestrator::solve_with_context(problem, ctx) {
        Ok(result) => result,
        Err(err) => TargetSolveResult::from_solver_error(err),
    }
}
```

---

## 8. File 別仕様: types

### 8.1 `types/ids.rs`

関数・型:

```rust
pub struct VariableId(pub u32);
pub struct RelationId(pub u32);
pub struct BlockId(pub u32);
pub struct PackageId(pub u32);
pub struct Hash(pub [u8; 32]);

pub fn fresh_variable_id(counter: &mut IdCounter) -> VariableId;
pub fn fresh_relation_id(counter: &mut IdCounter) -> RelationId;
pub fn stable_id_from_name(name: &str, namespace: &str) -> StableId;
```

各関数:

```text
fresh_variable_id:
    入力: mutable counter
    処理: counter を1増やし、新しい VariableId を返す
    出力: VariableId

stable_id_from_name:
    入力: name, namespace
    処理: canonical UTF-8 bytes を hash し stable id を作る
    出力: StableId
```

### 8.2 `types/rational.rs`

関数:

```rust
pub fn new_q(num: BigInt, den: BigInt) -> RationalQ;
pub fn normalize_q(q: RationalQ) -> RationalQ;
pub fn add_q(a: &RationalQ, b: &RationalQ) -> RationalQ;
pub fn sub_q(a: &RationalQ, b: &RationalQ) -> RationalQ;
pub fn mul_q(a: &RationalQ, b: &RationalQ) -> RationalQ;
pub fn div_q(a: &RationalQ, b: &RationalQ) -> Result<RationalQ, DivisionByZero>;
pub fn bit_height_q(q: &RationalQ) -> usize;
```

疑似コード:

```rust
fn normalize_q(q):
    if q.den == 0: error
    if q.num == 0: return 0/1
    if q.den < 0: q.num=-q.num; q.den=-q.den
    g = gcd(abs(q.num), q.den)
    return (q.num/g)/(q.den/g)
```

### 8.3 `types/monomial.rs`

関数:

```rust
pub fn normalize_monomial(entries: Vec<(VariableId,u32)>) -> Monomial;
pub fn monomial_mul(a: &Monomial, b: &Monomial) -> Monomial;
pub fn monomial_div(a: &Monomial, b: &Monomial) -> Option<Monomial>;
pub fn monomial_degree(m: &Monomial) -> u32;
pub fn monomial_variables(m: &Monomial) -> BTreeSet<VariableId>;
pub fn compare_monomial(a: &Monomial, b: &Monomial, order: &MonomialOrder) -> Ordering;
```

### 8.4 `types/polynomial.rs`

関数:

```rust
pub fn normalize_poly(p: SparsePolynomialQ) -> SparsePolynomialQ;
pub fn poly_add(a: &SparsePolynomialQ, b: &SparsePolynomialQ) -> SparsePolynomialQ;
pub fn poly_sub(a: &SparsePolynomialQ, b: &SparsePolynomialQ) -> SparsePolynomialQ;
pub fn poly_mul(a: &SparsePolynomialQ, b: &SparsePolynomialQ) -> SparsePolynomialQ;
pub fn poly_scale(a: &SparsePolynomialQ, c: &RationalQ) -> SparsePolynomialQ;
pub fn poly_derivative(a: &SparsePolynomialQ, v: VariableId) -> SparsePolynomialQ;
pub fn poly_variables(a: &SparsePolynomialQ) -> BTreeSet<VariableId>;
pub fn poly_total_degree(a: &SparsePolynomialQ) -> u32;
pub fn poly_monomial_count(a: &SparsePolynomialQ) -> usize;
pub fn clear_denominators_primitive(a: &SparsePolynomialQ) -> SparsePolynomialQ;
pub fn substitute_poly(a: &SparsePolynomialQ, subst: &SubstitutionMap) -> SparsePolynomialQ;
```

疑似コード:

```rust
fn normalize_poly(p):
    map = BTreeMap<Monomial, RationalQ>()
    for term in p.terms:
        if term.coeff != 0:
            map[normalize_monomial(term.monomial)] += term.coeff
    terms = []
    for (m,c) in map ordered by monomial_order:
        if c != 0: terms.push(TermQ{c,m})
    return SparsePolynomialQ{terms, hash=hash_terms(terms)}
```

### 8.5 `types/univariate.rs`

関数:

```rust
pub fn normalize_univariate(p: UniPolynomialQ) -> UniPolynomialQ;
pub fn degree_uni(p: &UniPolynomialQ) -> Option<usize>;
pub fn derivative_uni(p: &UniPolynomialQ) -> UniPolynomialQ;
pub fn gcd_uni(a: &UniPolynomialQ, b: &UniPolynomialQ) -> UniPolynomialQ;
pub fn squarefree_part_uni(p: &UniPolynomialQ) -> UniPolynomialQ;
pub fn eval_uni_q(p: &UniPolynomialQ, x: &RationalQ) -> RationalQ;
```

### 8.6 `types/matrix.rs`

型:

```rust
pub struct SparseMatrixQ { ... }
pub struct SparseMatrixFp { ... }
pub struct DenseMatrixFp { ... }
pub struct VectorQ { ... }
pub struct VectorFp { ... }
```

関数:

```rust
pub fn matrix_shape<M>(m: &M) -> (usize, usize);
pub fn matrix_density<M>(m: &M) -> RationalQ;
pub fn hash_matrix<M>(m: &M) -> Hash;
```

### 8.7 `types/interval.rs`

型・関数:

```rust
pub struct RationalInterval { pub lo: RationalQ, pub hi: RationalQ }

pub fn interval_new(lo: RationalQ, hi: RationalQ) -> Result<RationalInterval, IntervalError>;
pub fn interval_contains_q(i: &RationalInterval, x: &RationalQ) -> bool;
pub fn interval_disjoint(a: &RationalInterval, b: &RationalInterval) -> bool;
```

---

## 9. File 別仕様: problem

### 9.1 `problem/input.rs`

型:

```rust
pub struct RationalTargetProblem { ... }
pub struct VariableRoleRecord { ... }
```

関数:

```rust
pub fn make_problem(
    variables: Vec<VariableId>,
    target: VariableId,
    equations: Vec<SparsePolynomialQ>,
    semantics: Vec<RealConstraintEncoding>,
) -> RationalTargetProblem;
```

処理:

```text
1. 入力 vector を保持する。
2. hash を計算する。
3. ここでは canonicalization しない。
```

### 9.2 `problem/semantic.rs`

型:

```rust
pub enum RealConstraintKind { NonNegative, Positive, NonZero, BranchChoice, Other }
pub struct RealConstraintEncoding { ... }
```

関数:

```rust
pub fn register_slack_encoding(kind, encoded_relation_ids, slack_variables) -> RealConstraintEncoding;
pub fn semantic_relations(sem: &[RealConstraintEncoding]) -> BTreeSet<RelationId>;
pub fn verify_semantic_references(sem: &[RealConstraintEncoding], relations: &[RelationId]) -> Result<(), InvalidInput>;
```

### 9.3 `problem/validate.rs`

関数:

```rust
pub fn validate_input(problem: RationalTargetProblem) -> Result<ValidatedProblem, SolverError>;
```

疑似コード:

```rust
fn validate_input(problem):
    if problem.target not in problem.variables:
        return Err(InvalidInput("target not declared"))

    for f in problem.equations:
        if !is_finite_sparse_polynomial(f):
            return Err(InvalidInput("non-finite polynomial"))
        if !all_coefficients_are_rational(f):
            return Err(InvalidInput("non-rational coefficient"))

    verify_semantic_references(problem.semantic_encodings, relation_ids(problem.equations))?

    return ValidatedProblem(problem)
```

出力:

```text
ValidatedProblem
```

禁止:

```text
- target が coordinate role だから拒否する。
- branch/slack があるから拒否する。
- 幾何名がないから拒否する。
```

### 9.4 `problem/canonicalize.rs`

関数:

```rust
pub fn canonicalize_system(validated: ValidatedProblem) -> Result<CanonicalSystemQ, SolverError>;
pub fn canonicalize_relation(id: RelationId, p: SparsePolynomialQ) -> CanonicalRelationQ;
pub fn canonical_variable_order(vars: &[VariableId], target: VariableId) -> VariableOrder;
```

疑似コード:

```rust
fn canonicalize_system(validated):
    order = canonical_variable_order(validated.variables, validated.target)
    relations = []
    for (id, p) in enumerate(validated.equations):
        q = clear_denominators_primitive(p)
        q = normalize_poly(q)
        if q == 0:
            record_zero_relation_removed(id)
            continue
        if q is nonzero constant:
            return empty_or_invalid_by_semantics(id)
        relations.push(CanonicalRelationQ{id, polynomial=q, source=Input})
    return CanonicalSystemQ{variables, target, relations, order, semantic_encodings, hash}
```

禁止:

```text
- 因数分解して勝手に component を選ぶ。
- 分母を消した事実を semantic/certificate なしに忘れる。
- geometry label に基づいて式を変形する。
```

### 9.5 `problem/context.rs`

型:

```rust
pub struct SolverContext {
    pub options: SolverOptions,
    pub id_counter: IdCounter,
    pub hash_config: HashConfig,
    pub resource_meter: ResourceMeter,
    pub diagnostics: Vec<DiagnosticRecord>,
}
```

関数:

```rust
pub fn new_context(options: SolverOptions) -> SolverContext;
pub fn check_resource(ctx: &mut SolverContext, stage: StageId) -> Result<(), SolverError>;
pub fn push_diagnostic(ctx: &mut SolverContext, diag: DiagnosticRecord);
```

---

## 10. File 別仕様: algebra

### 10.1 `algebra/monomial_order.rs`

関数:

```rust
pub fn elimination_order(eliminate: &[VariableId], keep: &[VariableId]) -> MonomialOrder;
pub fn grevlex_order(vars: &[VariableId]) -> MonomialOrder;
pub fn lex_order(vars: &[VariableId]) -> MonomialOrder;
pub fn block_order(blocks: Vec<Vec<VariableId>>) -> MonomialOrder;
```

`elimination_order(Y,Z)` は、任意の `y∈Y` が任意の `z∈Z` より大きい order を返す。

### 10.2 `algebra/polynomial_ops.rs`

`types/polynomial.rs` の低レベル関数を使い、代数処理用の高レベル操作を提供する。

関数:

```rust
pub fn leading_term(p: &SparsePolynomialQ, order: &MonomialOrder) -> Option<TermQ>;
pub fn s_polynomial(f: &SparsePolynomialQ, g: &SparsePolynomialQ, order: &MonomialOrder) -> SparsePolynomialQ;
pub fn reduce_by_set(f: &SparsePolynomialQ, gs: &[SparsePolynomialQ], order: &MonomialOrder) -> ReductionResult;
pub fn content_primitive_part(f: &SparsePolynomialQ) -> (RationalQ, SparsePolynomialQ);
```

### 10.3 `algebra/modular.rs`

関数:

```rust
pub fn choose_prime_avoiding_denominators(polys: &[SparsePolynomialQ], seed: u64) -> Prime;
pub fn reduce_q_to_fp(p: &SparsePolynomialQ, prime: Prime) -> SparsePolynomialFp;
pub fn lift_fp_coeff(c: Fp, prime: Prime) -> IntegerRepresentative;
```

疑似コード:

```rust
fn choose_prime_avoiding_denominators(polys, seed):
    p = deterministic_prime_stream(seed)
    while p divides any denominator or leading coefficient forbidden set:
        p = next_prime(p)
    return p
```

### 10.4 `algebra/crt.rs`

関数:

```rust
pub fn crt_combine(a_mod_m, b_mod_n) -> ModInteger;
pub fn crt_vector_combine(v1, mod1, v2, mod2) -> ModVector;
```

### 10.5 `algebra/rational_reconstruction.rs`

関数:

```rust
pub fn reconstruct_rational(a_mod_m: ModInteger, modulus: BigInt, height_bound: Option<usize>) -> Option<RationalQ>;
pub fn reconstruct_polynomial(mod_poly_data, modulus) -> Option<SparsePolynomialQ>;
```

### 10.6 `algebra/sparse_matrix.rs`

関数:

```rust
pub fn build_sparse_matrix_fp(rows: Vec<SparseRowFp>, ncols: usize) -> SparseMatrixFp;
pub fn row_echelon_sparse_fp(m: &SparseMatrixFp, prime: Prime) -> EchelonResultFp;
pub fn nullspace_sparse_fp(m: &SparseMatrixFp, prime: Prime) -> Vec<VectorFp>;
pub fn rank_sparse_fp(m: &SparseMatrixFp, prime: Prime) -> usize;
```

### 10.7 `algebra/linear_solve.rs`

関数:

```rust
pub fn solve_homogeneous_modular(matrix_builder: MatrixBuilder, plan: ModularSolvePlan) -> ModularNullspaceResult;
pub fn solve_inhomogeneous_modular(matrix_builder: MatrixBuilder, rhs: VectorQ, plan: ModularSolvePlan) -> ModularSolveResult;
```

処理:

```text
1. deterministic prime sequence を選ぶ。
2. 各 prime で行列を構成する。
3. rank/nullspace を計算する。
4. rank profile が安定するまで prime を増やす。
5. CRT + rational reconstruction を行う。
6. Q 上の exact check を呼ぶ側に返す。
```

### 10.8 `algebra/normal_form.rs`

関数:

```rust
pub fn normal_form(p: &SparsePolynomialQ, basis: &[SparsePolynomialQ], order: &MonomialOrder) -> SparsePolynomialQ;
pub fn verify_membership_by_certificate(g: &SparsePolynomialQ, cert: &MembershipCertificate, relations: &[SparsePolynomialQ]) -> bool;
```

`verify_membership_by_certificate` の疑似コード:

```rust
fn verify_membership_by_certificate(g, cert, relations):
    sum = 0
    for term in cert.combination_terms:
        sum += term.multiplier * relations[term.relation_id]
    return normalize_poly(sum - g) == 0
```

### 10.9 `algebra/groebner.rs`

関数:

```rust
pub fn groebner_elimination_basis(
    relations: &[SparsePolynomialQ],
    order: &MonomialOrder,
    options: GroebnerOptions,
) -> Result<GroebnerBasisResult, SolverError>;

pub fn extract_elimination_generators(
    basis: &[SparsePolynomialQ],
    keep_variables: &BTreeSet<VariableId>,
) -> Vec<SparsePolynomialQ>;
```

制約:

```text
- この file は local block の target/separator elimination のために使う。
- global coordinate-first production result を作るために使ってはならない。
```

### 10.10 `algebra/f4.rs`

関数:

```rust
pub fn f4_reduce_batch(
    reducers: &[SparsePolynomialQ],
    targets: &[SparsePolynomialQ],
    order: &MonomialOrder,
    options: F4Options,
) -> Result<F4BatchReductionResult, SolverError>;

pub fn f4_elimination_local(
    relations: &[SparsePolynomialQ],
    eliminate: &[VariableId],
    keep: &[VariableId],
    options: F4Options,
) -> Result<LocalEliminationResult, SolverError>;
```

出力:

```text
LocalEliminationResult:
    generators in Q[keep]
    membership/normal-form certificates
    matrix size trace
```

### 10.11 `algebra/elimination.rs`

関数:

```rust
pub fn eliminate_to_keep_variables(
    relations: &[SparsePolynomialQ],
    eliminate: &[VariableId],
    keep: &[VariableId],
    strategy: EliminationStrategy,
    ctx: &mut SolverContext,
) -> Result<EliminationResult, SolverError>;
```

処理:

```text
1. keep と eliminate が disjoint であることを確認する。
2. strategy に従い Groebner / F4 / relation search / resultant を呼ぶ。
3. 結果 relation が Q[keep] に属することを確認する。
4. certificate を付ける。
```

### 10.12 `algebra/resultant.rs`

関数:

```rust
pub fn support_sets(polys: &[SparsePolynomialQ]) -> Vec<MonomialSupport>;
pub fn build_sparse_resultant_template(input: ResultantInput) -> Result<ResultantTemplate, SolverError>;
pub fn compute_resultant_relation(template: &ResultantTemplate, options: ModularOptions) -> Result<ResultantRelation, SolverError>;
pub fn verify_resultant_certificate(cert: &SparseResultantCertificate) -> bool;
```

### 10.13 `algebra/interpolation.rs`

関数:

```rust
pub fn choose_specialization_points(vars: &[VariableId], count: usize, prime: Prime) -> Vec<SpecializationPoint>;
pub fn specialize_polynomials(polys: &[SparsePolynomialQ], point: &SpecializationPoint) -> Vec<SparsePolynomialQ>;
pub fn interpolate_sparse_coefficients(samples: &[SpecializedRelation]) -> Result<SparsePolynomialQ, SolverError>;
pub fn verify_interpolated_relation(relation: &SparsePolynomialQ, certificate: &InterpolationCertificate) -> bool;
```

### 10.14 `algebra/quotient.rs`

関数・trait:

```rust
pub trait TargetQuotientHandle {
    fn basis_id(&self) -> BasisHandleId;
    fn basis_size(&self) -> usize;
    fn basis_scope(&self) -> BasisScope;
    fn normal_form(&self, p: &SparsePolynomialQ) -> Result<VectorQ, SolverError>;
    fn multiply_by_variable(&self, v: &VectorQ, var: VariableId) -> Result<VectorQ, SolverError>;
    fn no_coordinate_solution_export(&self) -> bool;
}

pub fn build_target_relevant_quotient_handle(input: QuotientHandleInput) -> Result<Box<dyn TargetQuotientHandle>, SolverError>;
```

禁止:

```text
- coordinate roots を返す API
- full coordinate RUR を返す API
- target-unrelated full quotient basis を公開する API
```

### 10.15 `algebra/krylov.rs`

関数:

```rust
pub fn block_krylov_sequence(handle: &dyn TargetQuotientHandle, target: VariableId, plan: KrylovPlan) -> Result<KrylovSequence, SolverError>;
pub fn recover_recurrence(seq: &KrylovSequence) -> Result<RecurrencePolynomial, SolverError>;
pub fn certify_krylov_coverage(seq: &KrylovSequence, recurrence: &RecurrencePolynomial, handle: &dyn TargetQuotientHandle) -> Result<CoverageCertificate, SolverError>;
pub fn verify_annihilator(handle: &dyn TargetQuotientHandle, poly: &UniPolynomialQ) -> Result<AnnihilatorCertificate, SolverError>;
```

### 10.16 `algebra/regular_chain.rs`

関数:

```rust
pub fn local_regular_chain_decomposition(input: RegularChainInput) -> Result<RegularChainDAG, SolverError>;
pub fn project_chain_to_variables(chain: &RegularChain, keep: &[VariableId]) -> Result<ProjectionGenerators, SolverError>;
pub fn combine_chain_projections(chains: &[ProjectionGenerators], semantics: UnionSemantics) -> Result<Vec<SparsePolynomialQ>, SolverError>;
```

### 10.17 `algebra/norm_trace.rs`

関数:

```rust
pub fn detect_explicit_tower(relations: &[SparsePolynomialQ], exported: &[VariableId]) -> Option<TowerDescription>;
pub fn norm_of_target_minus_expression(tower: &TowerDescription, target_expr: SparsePolynomialQ) -> Result<UniOrMultiPolynomialQ, SolverError>;
pub fn verify_norm_relation(tower: &TowerDescription, relation: &SparsePolynomialQ) -> bool;
```

### 10.18 `algebra/real_root.rs`

関数:

```rust
pub fn sturm_sequence(p: &UniPolynomialQ) -> Vec<UniPolynomialQ>;
pub fn isolate_real_roots_sturm(p: &UniPolynomialQ) -> Result<Vec<RealRootRecord>, SolverError>;
pub fn isolate_real_roots_descartes(p: &UniPolynomialQ) -> Result<Vec<RealRootRecord>, SolverError>;
```

### 10.19 `algebra/sign.rs`

関数:

```rust
pub fn sign_at_algebraic_root(poly: &UniPolynomialQ, root: &RealRootRecord) -> SignDetermination;
pub fn thom_encoding(poly: &UniPolynomialQ, root: &RealRootRecord) -> ThomEncoding;
```

---

## 11. File 別仕様: preprocess

### 11.1 `preprocess/compression.rs`

関数:

```rust
pub fn pre_kernel_compress(system: CanonicalSystemQ, ctx: &mut SolverContext) -> Result<CompressedSystemQ, SolverError>;
```

処理順:

```text
1. definitional elimination
2. linear affine elimination
3. binomial / monomial simplification
4. safe saturation for explicitly nonzero encodings
5. target-independent component marking
6. coefficient height and monomial count trace
```

疑似コード:

```rust
fn pre_kernel_compress(system, ctx):
    state = CompressionState::from(system)
    state = eliminate_definitional_variables(state, ctx)?
    state = eliminate_linear_affine_variables(state, ctx)?
    state = simplify_binomial_relations(state, ctx)?
    state = apply_explicit_saturations(state, ctx)?
    state = mark_target_independent_components(state, ctx)?
    return state.to_compressed_system()
```

禁止:

```text
- geometry name による rewrite
- expected answer による factor selection
- component semantics を記録しない factor split
```

### 11.2 `preprocess/definitional.rs`

目的: 明らかな定義変数を消す。

対象:

```text
y - p(X) = 0
c*y - p(X) = 0 where c ∈ Q\{0}
```

関数:

```rust
pub fn find_definitional_relations(system: &CanonicalSystemQ) -> Vec<DefinitionalCandidate>;
pub fn apply_definitional_elimination(state: CompressionState, candidates: &[DefinitionalCandidate], ctx: &mut SolverContext) -> Result<CompressionState, SolverError>;
```

疑似コード:

```rust
fn find_definitional_relations(system):
    candidates = []
    for relation in system.relations:
        for variable y in variables(relation):
            if relation is linear in y and coefficient(y) is nonzero rational constant:
                if y not target:
                    candidates.push(y = -rest/coefficient)
    sort candidates by deterministic cost score
    return candidates
```

注意:

```text
coefficient が variable-dependent の場合は definitional ではなく LinearAffine へ回す。
```

### 11.3 `preprocess/linear_affine.rs`

対象:

```text
a(X)*y + b(X) = 0
```

ただし `a(X)` が非ゼロであることを guard として記録できる場合のみ substitution する。非ゼロを証明できない場合は式を残す。

関数:

```rust
pub fn find_linear_affine_candidates(state: &CompressionState) -> Vec<LinearAffineCandidate>;
pub fn select_safe_affine_pivots(candidates: &[LinearAffineCandidate], policy: PivotPolicy) -> Vec<AffinePivot>;
pub fn eliminate_linear_affine_variables(state: CompressionState, ctx: &mut SolverContext) -> Result<CompressionState, SolverError>;
```

疑似コード:

```rust
fn eliminate_linear_affine_variables(state, ctx):
    loop:
        candidates = find_linear_affine_candidates(state)
        pivots = select_safe_affine_pivots(candidates, MarkowitzLikePolicy)
        if pivots.empty(): break
        pivot = pivots[0]
        if pivot.denominator_is_constant_nonzero:
            substitute directly
        else if pivot.denominator_has_recorded_nonzero_semantics:
            substitute and record denominator guard
        else:
            mark candidate rejected and continue
        if coefficient_height(state) > ctx.options.height_limit:
            return Err(FiniteResourceFailure)
    return state
```

### 11.4 `preprocess/binomial.rs`

目的: monomial/binomial 的な式の局所簡約を行う。

対象例:

```text
u - c*v = 0
u*v - c = 0
u^k - v = 0
```

関数:

```rust
pub fn detect_binomial_relations(state: &CompressionState) -> Vec<BinomialCandidate>;
pub fn simplify_binomial_relations(state: CompressionState, ctx: &mut SolverContext) -> Result<CompressionState, SolverError>;
```

制約:

```text
- union semantics を生む factor split はしない。
- 不可逆変形をするときは guard/certificate を残す。
```

### 11.5 `preprocess/saturation.rs`

関数:

```rust
pub fn apply_explicit_saturations(state: CompressionState, ctx: &mut SolverContext) -> Result<CompressionState, SolverError>;
pub fn is_explicit_nonzero_factor(factor: &SparsePolynomialQ, semantics: &[RealConstraintEncoding]) -> bool;
```

許可:

```text
A*s - 1 = 0 により A != 0 が明示されている場合、A による saturation を記録付きで使う。
```

### 11.6 `preprocess/independent.rs`

関数:

```rust
pub fn mark_target_independent_components(state: CompressionState, ctx: &mut SolverContext) -> Result<CompressionState, SolverError>;
pub fn compute_component_feasibility_obligations(components: &[Component]) -> Vec<FeasibilityObligation>;
```

規則:

```text
- target へ path を持たない component は candidate cover 構成から外してよい。
- exact image mode では feasibility obligation として残す。
- component が hard だから外すことは禁止。
```

---

## 12. File 別仕様: graph

### 12.1 `graph/hypergraph.rs`

関数:

```rust
pub fn build_relation_variable_hypergraph(system: &CompressedSystemQ) -> RelationVariableHypergraph;
pub fn connected_components(h: &RelationVariableHypergraph) -> Vec<HypergraphComponent>;
pub fn relation_variables(h: &RelationVariableHypergraph, r: RelationId) -> BTreeSet<VariableId>;
pub fn variable_relations(h: &RelationVariableHypergraph, v: VariableId) -> BTreeSet<RelationId>;
```

疑似コード:

```rust
fn build_relation_variable_hypergraph(system):
    h = empty
    for relation in system.relations:
        h.add_relation(relation.id)
        vars = poly_variables(relation.polynomial)
        for v in vars:
            h.add_variable(v)
            h.add_incidence(relation.id, v)
    assert every polynomial occurrence is represented
    return h
```

### 12.2 `graph/influence.rs`

関数:

```rust
pub fn build_target_influence_graph(h: &RelationVariableHypergraph, target: VariableId) -> TargetInfluenceGraph;
```

疑似コード:

```rust
fn build_target_influence_graph(h, target):
    component = BFS in bipartite graph from target variable node
    independent = all other components
    return TargetInfluenceGraph{target_component, independent_components}
```

### 12.3 `graph/weighted_primal.rs`

関数:

```rust
pub fn build_weighted_primal_graph(system: &CompressedSystemQ, influence: &TargetInfluenceGraph) -> WeightedPrimalGraph;
pub fn variable_weight(v: VariableId, system: &CompressedSystemQ) -> AlgebraicWeight;
pub fn edge_weight(u: VariableId, v: VariableId, relations: &[RelationId]) -> AlgebraicWeight;
```

重み:

```text
- variable degree participation
- monomial count contribution
- coefficient height contribution
- target distance
- linear eliminability
- occurrence count
```

### 12.4 `graph/separators.rs`

関数:

```rust
pub fn articulation_variable_candidates(g: &WeightedPrimalGraph) -> Vec<SeparatorCandidate>;
pub fn min_fill_separator_candidates(g: &WeightedPrimalGraph, target: VariableId) -> Vec<SeparatorCandidate>;
pub fn score_separator(candidate: &SeparatorCandidate, subgraph: &WeightedPrimalGraph, cost_model: &CostModel) -> SeparatorScore;
```

疑似コード:

```rust
fn score_separator(candidate, subgraph, cost_model):
    components = remove_separator(subgraph, candidate.vars)
    score = 0
    for comp in components:
        estimated_block = comp.vars ∪ candidate.vars
        score += estimate_local_projection_cost(estimated_block)
    score += separator_degree_penalty(candidate)
    score += coefficient_height_penalty(candidate)
    return score
```

### 12.5 `graph/tree_decomposition.rs`

関数:

```rust
pub fn build_target_rooted_decomposition(g: &WeightedPrimalGraph, target: VariableId, cost_model: &CostModel) -> DecompositionTree;
```

疑似コード:

```rust
fn decompose(subgraph, target_side):
    if subgraph.size <= options.max_direct_block_width:
        return leaf_block(subgraph)

    candidates = articulation_variable_candidates(subgraph)
               ∪ min_fill_separator_candidates(subgraph, target_side)
               ∪ bounded_min_cut_candidates(subgraph, target_side)

    scored = sort_by(score_separator)
    for cand in scored:
        if improves_estimated_cost(cand, subgraph):
            children = components_after_removing(cand)
            return node(separator=cand, children=map(decompose, children))

    return leaf_block(subgraph) // not unsupported
```

### 12.6 `graph/projection_dag.rs`

関数:

```rust
pub fn build_target_projection_dag(
    system: &CompressedSystemQ,
    influence: &TargetInfluenceGraph,
    decomposition: &DecompositionTree,
) -> Result<TargetProjectionDAG, SolverError>;

pub fn authorize_block_relations(block: &mut ProjectionBlock, system: &CompressedSystemQ) -> AuthorizationHash;
pub fn validate_projection_dag(dag: &TargetProjectionDAG, system: &CompressedSystemQ) -> Result<(), SolverError>;
```

規則:

```text
- 各 relation は原則1つの block に属する。
- relation duplication が必要な場合は DuplicationCertificate が必要。
- 各 block の projector は authorization_hash に含まれた relation だけ読める。
- no useful separator の場合は one large target block を作る。
```

### 12.7 `graph/metrics.rs`

関数:

```rust
pub fn structural_metrics(block: &ProjectionBlock, system: &CompressedSystemQ) -> StructuralMetrics;
pub fn estimate_local_quotient_rank(block: &ProjectionBlock) -> RankEstimate;
pub fn estimate_sparse_template_size(block: &ProjectionBlock) -> TemplateEstimate;
pub fn estimate_coefficient_growth(block: &ProjectionBlock) -> HeightEstimate;
```

---

## 13. File 別仕様: planner

### 13.1 `planner/cost_model.rs`

型:

```rust
pub struct CostModelWeights {
    pub matrix_size_weight: RationalQ,
    pub quotient_rank_weight: RationalQ,
    pub coefficient_height_weight: RationalQ,
    pub separator_degree_weight: RationalQ,
    pub certificate_cost_weight: RationalQ,
}
```

関数:

```rust
pub fn estimate_kernel_cost(block: &ProjectionBlock, kernel: KernelKind, probes: &ProbeResults) -> KernelCostEstimate;
pub fn compare_cost(a: &KernelCostEstimate, b: &KernelCostEstimate) -> Ordering;
```

比較は deterministic である。

### 13.2 `planner/probes.rs`

関数:

```rust
pub fn run_cost_probes(block: &ProjectionBlock, system: &CompressedSystemQ, ctx: &mut SolverContext) -> ProbeResults;
pub fn modular_rank_probe(block: &ProjectionBlock, primes: &[Prime]) -> RankProbeResult;
pub fn local_macaulay_size_probe(block: &ProjectionBlock) -> MacaulaySizeProbe;
pub fn mixed_support_probe(block: &ProjectionBlock) -> SparseSupportProbe;
```

注意:

```text
probe は planner の cost estimate にだけ使う。
正しさの証明には使わない。
```

### 13.3 `planner/admission.rs`

関数:

```rust
pub fn collect_kernel_admissions(block: &ProjectionBlock, ctx: &KernelContext) -> Vec<KernelAdmission>;
```

処理:

```text
1. すべての kernel の admit を呼ぶ。
2. UniversalTargetEliminationKernel は well-formed block なら必ず admissible。
3. admission false は runtime failure ではない。
4. admissions を hash 付きで返す。
```

### 13.4 `planner/kernel_plan.rs`

型:

```rust
pub struct KernelPlan {
    pub block_id: BlockId,
    pub declared_ladder: Vec<KernelExecutionPlan>,
    pub selected_first: KernelKind,
    pub admissions: Vec<KernelAdmission>,
    pub cost_estimates: Vec<KernelCostEstimate>,
    pub plan_hash: Hash,
}
```

`declared_ladder` は hidden fallback ではない。実行前に全て確定し、certificate に入る。

### 13.5 `planner/ladder.rs`

関数:

```rust
pub fn build_declared_ladder(admissions: &[KernelAdmission], costs: &[KernelCostEstimate]) -> Vec<KernelExecutionPlan>;
```

規則:

```text
- certificate available な kernel だけ ladder に入れる。
- coordinate-first kernel は存在してはならない。
- UniversalTargetEliminationKernel は最後の generic target-direct plan として入る。
- ladder の各 plan は resource bound と degree bound を明示する。
```

### 13.6 `planner/planner.rs`

関数:

```rust
pub fn plan_all_blocks(dag: &TargetProjectionDAG, system: &CompressedSystemQ, ctx: &mut SolverContext) -> Result<Vec<KernelPlan>, SolverError>;
```

疑似コード:

```rust
fn plan_all_blocks(dag, system, ctx):
    plans = []
    for block in postorder(dag):
        kctx = KernelContext::new(block, system, messages_from_children)
        probes = run_cost_probes(block, system, ctx)
        admissions = collect_kernel_admissions(block, kctx)
        costs = admissions.map(|a| estimate_kernel_cost(block, a.kind, probes))
        ladder = build_declared_ladder(admissions, costs)
        if ladder.empty():
            return Err(ImplementationBug("Universal kernel missing"))
        plans.push(KernelPlan{block_id, ladder, selected_first=ladder[0].kind, ...})
    return plans
```

---

## 14. File 別仕様: kernels 共通

### 14.1 `kernels/traits.rs`

trait:

```rust
pub trait TargetProjectionKernel {
    fn kind(&self) -> KernelKind;
    fn admit(&self, block: &ProjectionBlock, ctx: &KernelContext) -> KernelAdmission;
    fn plan(&self, admission: &KernelAdmission, ctx: &KernelContext) -> Result<KernelExecutionPlan, SolverError>;
    fn execute(&self, plan: &KernelExecutionPlan, ctx: &mut KernelContext) -> Result<ProjectionMessage, SolverError>;
    fn replay(&self, message: &ProjectionMessage, ctx: &KernelContext) -> ReplayResult;
}
```

`execute` 共通条件:

```text
入力:
    plan: planner が事前に宣言した実行計画
    ctx: block relation、child messages、resource meter

処理:
    target/separator relation を構成する。
    exact Q certificate を作る。
    local coordinate solution を出力しない。

出力:
    ProjectionMessage
```

### 14.2 `kernels/mod.rs`

関数:

```rust
pub fn all_kernels() -> Vec<Box<dyn TargetProjectionKernel>>;
pub fn kernel_by_kind(kind: KernelKind) -> Box<dyn TargetProjectionKernel>;
```

`all_kernels()` は次を返す。

```text
1. TargetUnivariateKernel
2. LinearAffineKernel
3. TargetRelationSearchKernel
4. SparseResultantProjectionKernel
5. TargetActionKrylovKernel
6. NormTraceProjectionKernel
7. RegularChainProjectionKernel
8. SpecializationInterpolationKernel
9. UniversalTargetEliminationKernel
```

---

## 15. Kernel: TargetUnivariateKernel

File: `kernels/target_univariate.rs`

### 15.1 Admission

```rust
pub fn admit_target_univariate(block: &ProjectionBlock, ctx: &KernelContext) -> KernelAdmission;
```

条件:

```text
block relation または child message relation の中に、variables(f) ⊆ {T} を満たす非零 relation が存在する。
```

### 15.2 Execute

```rust
pub fn execute_target_univariate(plan: &KernelExecutionPlan, ctx: &mut KernelContext) -> Result<ProjectionMessage, SolverError>;
```

疑似コード:

```rust
fn execute_target_univariate(plan, ctx):
    rels = []
    for r in ctx.available_relations():
        if variables(r) subset {target} and r != 0:
            rels.push(convert_to_univariate(r))

    if rels.empty():
        return Err(ImplementationBug("admission invalid"))

    support = primitive_lcm_squarefree(rels)
    cert = SourceMembershipCertificate(rels.source_ids)

    return ProjectionMessage{
        exported_variables: [target],
        relation_generators: [support_as_sparse],
        representation: PrincipalSupport,
        projection_strength: CandidateCoverStrong,
        certificate: cert,
        ...
    }
```

---

## 16. Kernel: LinearAffineKernel

File: `kernels/linear_affine.rs`

### 16.1 Admission

条件:

```text
block local variables Y can be eliminated by triangular affine substitutions,
leaving relations only in exported variables Z={T}+separators.
```

関数:

```rust
pub fn find_triangular_affine_order(block: &ProjectionBlock, ctx: &KernelContext) -> Option<AffineEliminationOrder>;
```

### 16.2 Execute

```rust
pub fn execute_linear_affine(plan: &KernelExecutionPlan, ctx: &mut KernelContext) -> Result<ProjectionMessage, SolverError>;
```

疑似コード:

```rust
fn execute_linear_affine(plan, ctx):
    order = plan.affine_order
    relations = ctx.block_relations.clone()
    substitutions = []

    for step in order:
        equation = choose_equation_linear_in(step.variable)
        (a,b) = split_as_a_x_plus_b(equation, step.variable)
        if a is constant nonzero:
            subst = x -> -b/a
        else if ctx.has_nonzero_guard(a):
            subst = x -> -b/a
            record_denominator_guard(a)
        else:
            return Err(ImplementationBug("unsafe affine pivot in plan"))
        relations = substitute_all(relations, subst)
        substitutions.push(subst)
        check_resource()

    exported_relations = []
    for r in relations:
        r = clear_denominators_primitive(r)
        if variables(r) subset exported_variables:
            exported_relations.push(r)
        else if r != 0:
            return Err(AlgorithmicHardCase("affine elimination incomplete"))

    cert = LinearAffineCertificate(substitutions, exported_relations)
    return ProjectionMessage{relation_generators: exported_relations, certificate: cert, ...}
```

---

## 17. Kernel: TargetRelationSearchKernel

File: `kernels/target_relation_search.rs`

この kernel は v4 の中心的な generic target-direct workhorse である。

### 17.1 数学的目的

局所 block の ideal を

```text
J = <f1,...,fr> ⊂ Q[Y,Z]
```

とする。ここで

```text
Y = eliminated local variables
Z = exported variables = {T}+separators
```

目的は、非零 relation

```text
g(Z) ∈ J ∩ Q[Z]
```

を、座標解を出さずに見つけることである。

### 17.2 基本方程式

未知係数を置く。

```text
g(Z) = Σ_{α∈A} c_α Z^α
q_i(Y,Z) = Σ_{β∈B_i} u_{i,β} YZ^β
```

次を満たす係数を探す。

```text
g(Z) = Σ_i q_i(Y,Z) f_i(Y,Z)
```

係数比較により線形方程式を得る。非自明な `c_α` が得られれば、`g ∈ J∩Q[Z]` の membership certificate も同時に得られる。

### 17.3 Admission

全 block に対して admission は可能だが、planner は cost estimate により実行順を決める。

```rust
pub fn admit_target_relation_search(block: &ProjectionBlock, ctx: &KernelContext) -> KernelAdmission;
```

返す情報:

```text
- exported variables Z
- eliminated variables Y
- initial degree bounds
- support strategy
- estimated matrix size
```

### 17.4 Execute

```rust
pub fn execute_target_relation_search(plan: &KernelExecutionPlan, ctx: &mut KernelContext) -> Result<ProjectionMessage, SolverError>;
```

疑似コード:

```rust
fn execute_target_relation_search(plan, ctx):
    J = ctx.local_relations_plus_child_messages()
    Z = ctx.exported_variables()
    Y = ctx.local_variables_minus(Z)

    for bound in plan.declared_degree_bounds:
        A = build_export_monomial_support(Z, bound, plan.support_strategy)
        B = build_multiplier_supports(J, Y, Z, bound, plan.multiplier_strategy)

        matrix_builder = build_membership_matrix_builder(J, A, B)
        ns = solve_homogeneous_modular(matrix_builder, plan.modular_plan)

        candidates = reconstruct_candidate_relations(ns, A, B)
        for cand in candidates sorted deterministically:
            if cand.g == 0:
                continue
            if variables(cand.g) not subset Z:
                continue
            if verify_membership_exact(cand.g, cand.qs, J):
                cert = MembershipCertificate(g=cand.g, qs=cand.qs)
                return ProjectionMessage{
                    exported_variables: Z,
                    relation_generators: [primitive(cand.g)],
                    representation: GeneratorSet,
                    projection_strength: CandidateCoverStrong,
                    certificate: cert,
                    cost_trace: matrix_builder.trace,
                }

    return Err(AlgorithmicHardCase{
        reason: "no relation found within declared bounds",
        matrix_trace: accumulated_trace,
    })
```

### 17.5 build_export_monomial_support

```rust
pub fn build_export_monomial_support(
    exported: &[VariableId],
    bound: DegreeBound,
    strategy: SupportStrategy,
) -> Vec<Monomial>;
```

戦略:

```text
DenseTotalDegree:
    all monomials in Z with total degree <= bound

SparseFromProjectionFootprint:
    monomials predicted from supports of local relations after eliminating Y

SpecializedInterpolationFootprint:
    monomials seen in specialized runs
```

### 17.6 build_multiplier_supports

```rust
pub fn build_multiplier_supports(
    relations: &[SparsePolynomialQ],
    eliminated: &[VariableId],
    exported: &[VariableId],
    bound: DegreeBound,
    strategy: MultiplierSupportStrategy,
) -> Vec<Vec<Monomial>>;
```

規則:

```text
q_i f_i の monomials が coefficient comparison set に入るように B_i を作る。
B_i は deterministic に並べる。
coefficient explosion が予測される場合は FiniteResourceFailure ではなく cost estimate を planner に返す。
execute 中に resource limit を超えた場合のみ FiniteResourceFailure。
```

### 17.7 Certificate

```yaml
TargetRelationSearchCertificate:
  exported_variables_hash: ""
  eliminated_variables_hash: ""
  export_support_hash: ""
  multiplier_support_hash: ""
  membership_matrix_hash: ""
  primes_used: []
  rational_reconstruction_hash: ""
  relation_hash: ""
  multipliers_hash: ""
  exact_identity_hash: "g - Σ q_i f_i = 0"
```

---

## 18. Kernel: SparseResultantProjectionKernel

File: `kernels/sparse_resultant.rs`

### 18.1 Admission

```rust
pub fn admit_sparse_resultant(block: &ProjectionBlock, ctx: &KernelContext) -> KernelAdmission;
```

条件:

```text
- eliminated variables Y and exported variables Z are known.
- support sets allow finite resultant/eliminant template estimate.
- estimated sparse template size is finite.
- exact verification method is available.
```

「sparse enough でなければ unsupported」ではない。admission false になるだけで、他の generic kernel が処理する。

### 18.2 Execute

```rust
pub fn execute_sparse_resultant(plan: &KernelExecutionPlan, ctx: &mut KernelContext) -> Result<ProjectionMessage, SolverError>;
```

疑似コード:

```rust
fn execute_sparse_resultant(plan, ctx):
    polys = ctx.local_relations_plus_child_messages()
    supports = support_sets(polys)
    template = build_sparse_resultant_template(supports, plan.template_options)?

    rel_mod = compute_template_determinant_or_null_relation(template, plan.modular_plan)?
    relation = reconstruct_polynomial(rel_mod)?

    if variables(relation) not subset ctx.exported_variables:
        return Err(ImplementationBug("resultant exported local variable"))

    cert = verify_resultant_or_membership(relation, template, polys)?

    return ProjectionMessage{
        relation_generators: [primitive(relation)],
        representation: SparseResultantMatrix,
        projection_strength: CandidateCoverStrong,
        certificate: cert,
        ...
    }
```

---

## 19. Kernel: TargetActionKrylovKernel

File: `kernels/action_krylov.rs`

### 19.1 目的

有限 rank の target-relevant quotient/action が安く作れる場合、multiplication-by-target operator の annihilating polynomial を計算して target support を得る。

### 19.2 Admission

```rust
pub fn admit_action_krylov(block: &ProjectionBlock, ctx: &KernelContext) -> KernelAdmission;
```

条件:

```text
- local quotient/action handle の rank estimate が有限。
- normal form が計算可能。
- handle が coordinate roots / full coordinate RUR を公開しない。
- coverage certificate を構成できる見込みがある。
```

### 19.3 Execute

```rust
pub fn execute_action_krylov(plan: &KernelExecutionPlan, ctx: &mut KernelContext) -> Result<ProjectionMessage, SolverError>;
```

疑似コード:

```rust
fn execute_action_krylov(plan, ctx):
    handle = build_target_relevant_quotient_handle(plan.quotient_plan, ctx)?
    if !handle.no_coordinate_solution_export():
        return Err(ImplementationBug("coordinate-exporting handle"))

    seq = block_krylov_sequence(handle, ctx.target, plan.krylov_plan)?
    recurrence = recover_recurrence(seq)?
    coverage = certify_krylov_coverage(seq, recurrence, handle)?

    if !coverage.valid:
        return Err(CertificateDesignGap("krylov coverage missing"))

    annihilator = recurrence.to_univariate(ctx.target)
    ann_cert = verify_annihilator(handle, annihilator)?

    return ProjectionMessage{
        exported_variables: [ctx.target] plus ctx.separators_if_symbolic,
        relation_generators: [annihilator_as_sparse],
        representation: QuotientAction,
        projection_strength: CandidateCoverStrong,
        certificate: TargetActionKrylovCertificate(coverage, ann_cert),
    }
```

### 19.4 Coverage 問題

単一 Krylov 列は eigenvalue を見落とす可能性がある。そのため、以下のいずれかが必要である。

```text
- deterministic basis probe coverage
- block Wiedemann rank/degree proof
- trace power certificate
- verified characteristic support comparison
- S(M_T)=0 plus proof that all target-relevant eigenvalues are included
```

coverage が証明できない場合、candidate polynomial を返してはならない。

---

## 20. Kernel: UniversalTargetEliminationKernel

File: `kernels/universal_elimination.rs`

### 20.1 位置づけ

この kernel は fallback ではない。有理数係数多項式系全体を入力範囲にするために必要な、計画済み generic target/separator projection kernel である。

### 20.2 Admission

```rust
pub fn admit_universal_elimination(block: &ProjectionBlock, ctx: &KernelContext) -> KernelAdmission;
```

条件:

```text
block が Q-polynomial relations を持つなら true。
```

### 20.3 Execute

```rust
pub fn execute_universal_elimination(plan: &KernelExecutionPlan, ctx: &mut KernelContext) -> Result<ProjectionMessage, SolverError>;
```

疑似コード:

```rust
fn execute_universal_elimination(plan, ctx):
    Z = ctx.exported_variables()
    Y = ctx.local_variables_minus(Z)
    J = ctx.local_relations_plus_child_messages()

    order = elimination_order(Y, Z)

    result = algebra::elimination::eliminate_to_keep_variables(
        J, Y, Z, plan.elimination_strategy, ctx
    )?

    gens = []
    for g in result.generators:
        if g != 0 and variables(g) subset Z:
            gens.push(primitive(g))

    if gens.empty():
        if certify_nonfinite_projection(J, Z):
            return Err(CertifiedNonFiniteTargetImage)
        else:
            return Err(AlgorithmicHardCase("no exported relation"))

    cert = result.certificate
    verify_every_generator_exact(gens, cert, J)?

    return ProjectionMessage{
        exported_variables: Z,
        relation_generators: gens,
        representation: GeneratorSet,
        projection_strength: ExactProjectionIdeal or CandidateCoverStrong,
        certificate: cert,
        cost_trace: result.cost_trace,
    }
```

### 20.4 内部戦略

`UniversalTargetEliminationKernel` は次の内部戦略を planner が選ぶ。

```text
- EliminationGroebnerLocal
- F4EliminationLocal
- TargetRelationSearchEscalated
- ResultantIfSquareOrOverdetermined
- SpecializeProjectInterpolateVerify
```

禁止:

```text
- coordinate solution enumeration
- full coordinate RUR
- global QE/CAD
- geometry-specific branch
- runtime hidden fallback
```

---

## 21. Kernel: RegularChainProjectionKernel

File: `kernels/regular_chain_projection.rs`

### 21.1 Admission

```rust
pub fn admit_regular_chain_projection(block: &ProjectionBlock, ctx: &KernelContext) -> KernelAdmission;
```

条件:

```text
- triangular pattern が見える。
- component/guard/projection semantics を保つ必要がある。
- compact ComponentDAG を作れる見込みがある。
```

### 21.2 Execute

```rust
pub fn execute_regular_chain_projection(plan: &KernelExecutionPlan, ctx: &mut KernelContext) -> Result<ProjectionMessage, SolverError>;
```

疑似コード:

```rust
fn execute_regular_chain_projection(plan, ctx):
    chains = local_regular_chain_decomposition(ctx.local_relations, plan.chain_options)?
    component_messages = []
    for chain in chains:
        projected = project_chain_to_variables(chain, ctx.exported_variables)?
        cert = verify_chain_projection(chain, projected)?
        component_messages.push((projected, cert))

    combined = combine_component_relations(component_messages, union_semantics)

    return ProjectionMessage{
        relation_generators: combined.generators,
        representation: TriangularChain,
        projection_strength: CandidateCoverStrong,
        certificate: combined.certificate,
    }
```

---

## 22. Kernel: NormTraceProjectionKernel

File: `kernels/norm_trace_projection.rs`

### 22.1 Admission

```rust
pub fn admit_norm_trace_projection(block: &ProjectionBlock, ctx: &KernelContext) -> KernelAdmission;
```

条件:

```text
relations が明示的な有限代数塔を定義する。
例: α satisfies p(α)=0, T = r(α,Z)
```

これは幾何名ではなく、式の代数形で判定する。

### 22.2 Execute

```rust
pub fn execute_norm_trace_projection(plan: &KernelExecutionPlan, ctx: &mut KernelContext) -> Result<ProjectionMessage, SolverError>;
```

疑似コード:

```rust
fn execute_norm_trace_projection(plan, ctx):
    tower = detect_explicit_tower(ctx.local_relations, ctx.exported_variables)?
    relation = norm_of_target_minus_expression(tower, plan.target_expression)?
    if !verify_norm_relation(tower, relation):
        return Err(CertificateDesignGap("norm relation not verified"))
    return ProjectionMessage{relation_generators:[relation], representation:NormTraceTower, ...}
```

---

## 23. Kernel: SpecializationInterpolationKernel

File: `kernels/specialization_interpolation.rs`

### 23.1 目的

separator が複数ある場合、`g(T,u1,...,uτ)` を直接構成すると係数膨張が起きる。そこで separator を一時的に特殊化し、target relation を計算し、係数を補間し、最後に Q 上で検証する。

### 23.2 Admission

```rust
pub fn admit_specialization_interpolation(block: &ProjectionBlock, ctx: &KernelContext) -> KernelAdmission;
```

条件:

```text
- exported variables Z のうち target 以外の separator が存在する。
- specialization 後の target-only relation 計算が安くなる見込みがある。
- interpolation support bound が宣言できる。
- 最後に exact Q verification が可能。
```

### 23.3 Execute

```rust
pub fn execute_specialization_interpolation(plan: &KernelExecutionPlan, ctx: &mut KernelContext) -> Result<ProjectionMessage, SolverError>;
```

疑似コード:

```rust
fn execute_specialization_interpolation(plan, ctx):
    T = ctx.target
    U = ctx.exported_variables - {T}
    samples = []

    for point in choose_specialization_points(U, plan.sample_count, plan.primes):
        specialized_ctx = specialize_context(ctx, U, point)
        local_plan = plan.inner_target_only_plan
        msg = execute_declared_inner_kernel(local_plan, specialized_ctx)?
        samples.push((point, msg.relation_generators[0]))

    relation = interpolate_sparse_coefficients(samples, plan.interpolation_support)?

    if variables(relation) not subset ctx.exported_variables:
        return Err(ImplementationBug("interpolated relation has local variables"))

    cert = verify_interpolated_relation_by_membership_or_elimination(relation, ctx)?

    return ProjectionMessage{
        relation_generators: [relation],
        representation: SpecializationInterpolation,
        projection_strength: CandidateCoverStrong,
        certificate: cert,
    }
```

重要:

```text
specialization/interpolation は候補生成である。
正しさは最後の exact Q verification によってのみ認める。
```

---

## 24. Projection message の合成

### 24.1 `compose/message.rs`

型:

```rust
pub struct MessageIdeal {
    pub variables: Vec<VariableId>,
    pub relations: Vec<SparsePolynomialQ>,
    pub source_packages: Vec<PackageId>,
}
```

関数:

```rust
pub fn message_to_relations(message: &ProjectionMessage) -> Vec<SparsePolynomialQ>;
pub fn merge_messages(messages: &[ProjectionMessage]) -> MessageIdeal;
```

### 24.2 `compose/compose.rs`

関数:

```rust
pub fn compose_projection_messages(
    dag: &TargetProjectionDAG,
    messages: Vec<ProjectionMessage>,
    ctx: &mut SolverContext,
) -> Result<ComposedProjection, SolverError>;
```

疑似コード:

```rust
fn compose_projection_messages(dag, messages, ctx):
    current = messages indexed by block
    for node in postorder_to_root(dag):
        incoming = child_messages(node)
        local_msg = current[node]
        merged = merge_messages(incoming ∪ local_msg)
        if merged.variables subset {target}:
            attach_to_node(node, merged)
        else:
            eliminated = eliminate_separators(merged, node.parent_separator, ctx)?
            attach_to_parent(node, eliminated)
    return root_composed_projection
```

### 24.3 `compose/separator_elimination.rs`

関数:

```rust
pub fn eliminate_remaining_separators(
    ideal: MessageIdeal,
    keep: &[VariableId],
    ctx: &mut SolverContext,
) -> Result<ProjectionMessage, SolverError>;
```

処理:

```text
1. Y = ideal.variables - keep
2. Z = keep
3. 新しい pseudo block を作る。ただし source は message relation のみ。
4. planner を呼び、target-direct kernel で Y を消去する。
5. 結果 relation は Q[Z] に属する。
```

禁止:

```text
- 元の全 coordinate system を再構築して global solve する。
- separator elimination に local coordinate variables を勝手に戻す。
```

### 24.4 `compose/final_support.rs`

関数:

```rust
pub fn build_global_support_polynomial(
    composed: ComposedProjection,
    target: VariableId,
    ctx: &mut SolverContext,
) -> Result<UniPolynomialQ, SolverError>;
```

疑似コード:

```rust
fn build_global_support_polynomial(composed, target, ctx):
    rels = composed.root_relations
    target_only = []
    for r in rels:
        if variables(r) subset {target}:
            target_only.push(convert_to_univariate(r))

    if target_only.empty():
        if certify_nonfinite_target_image(composed):
            return Err(CertifiedNonFiniteTargetImage)
        else:
            return Err(AlgorithmicHardCase("no target-only support after composition"))

    S = primitive_lcm(target_only)
    return normalize_univariate(S)
```

---

## 25. Support verification

### 25.1 `verify/certificates.rs`

型:

```rust
pub enum KernelCertificate {
    SourceRelation(SourceRelationCertificate),
    Membership(MembershipCertificate),
    NormalForm(NormalFormCertificate),
    SparseResultant(SparseResultantCertificate),
    TargetAction(TargetActionKrylovCertificate),
    RegularChain(RegularChainProjectionCertificate),
    NormTrace(NormTraceCertificate),
    SpecializationInterpolation(InterpolationCertificate),
    Composite(CompositeCertificate),
}
```

### 25.2 `verify/verify_message.rs`

関数:

```rust
pub fn verify_projection_message(message: &ProjectionMessage, ctx: &KernelContext) -> Result<(), SolverError>;
```

疑似コード:

```rust
fn verify_projection_message(message, ctx):
    assert message.exported_variables subset ctx.allowed_exported_variables
    for g in message.relation_generators:
        assert variables(g) subset message.exported_variables

    match message.certificate:
        Membership(cert) => verify_membership_by_certificate(g, cert, ctx.source_relations)
        SparseResultant(cert) => verify_resultant_certificate(cert)
        TargetAction(cert) => verify_target_action_certificate(cert)
        RegularChain(cert) => verify_regular_chain_projection(cert)
        NormTrace(cert) => verify_norm_trace_certificate(cert)
        SpecializationInterpolation(cert) => verify_interpolation_certificate(cert)
        Composite(cert) => verify_composite_certificate(cert)

    return Ok
```

### 25.3 `verify/verify_support.rs`

関数:

```rust
pub fn verify_global_support(
    support: &UniPolynomialQ,
    composed: &ComposedProjection,
    run_context: &SolverContext,
) -> Result<GlobalSupportCertificate, SolverError>;
```

目的:

```text
S(T) が全ての真の target fiber 上で消えることを証明する。
```

疑似コード:

```rust
fn verify_global_support(S, composed, ctx):
    support_sparse = univariate_to_sparse(S)
    if support_sparse equals lcm/product of verified target-only root relations:
        return CompositeCertificate(from target-only messages)

    if membership certificate in composed message ideal exists:
        verify membership
        return certificate

    return Err(CertificateDesignGap("support verification missing"))
```

### 25.4 `verify/replay.rs`

関数:

```rust
pub fn replay_run_certificate(result: &TargetSolveResult, problem: &RationalTargetProblem) -> ReplayResult;
```

処理:

```text
1. input hash を確認する。
2. canonicalization hash を再計算する。
3. DAG hash を確認する。
4. 各 ProjectionMessage を replay する。
5. support verification を replay する。
6. root isolation hash を replay する。
```

### 25.5 `verify/run_certificate.rs`

型:

```rust
pub struct CoreRunCertificate {
    pub input_hash: Hash,
    pub canonical_system_hash: Hash,
    pub target_variable: VariableId,
    pub compression_hash: Hash,
    pub hypergraph_hash: Hash,
    pub target_projection_dag_hash: Hash,
    pub kernel_plan_hashes: Vec<Hash>,
    pub projection_message_hashes: Vec<Hash>,
    pub global_support_hash: Option<Hash>,
    pub squarefree_support_hash: Option<Hash>,
    pub root_isolation_hash: Option<Hash>,
    pub decoded_candidate_hash: Option<Hash>,
    pub fiber_classification_hash: Option<Hash>,
    pub invariants: CoreInvariantFlags,
}

pub struct CoreInvariantFlags {
    pub no_geometry_dispatch: bool,
    pub no_problem_id_dispatch: bool,
    pub no_expected_answer_dispatch: bool,
    pub no_full_coordinate_solution_set: bool,
    pub no_full_coordinate_rur: bool,
    pub no_qe_cad: bool,
    pub exact_q_verification: bool,
    pub no_hidden_fallback: bool,
}
```

---

## 26. Root isolation と candidate decode

### 26.1 `roots/squarefree.rs`

関数:

```rust
pub fn squarefree_support(p: &UniPolynomialQ) -> Result<UniPolynomialQ, SolverError>;
```

疑似コード:

```rust
fn squarefree_support(p):
    if p == 0: return Err(AlgorithmicHardCase("zero support"))
    d = derivative_uni(p)
    g = gcd_uni(p, d)
    return normalize_univariate(p / g)
```

### 26.2 `roots/isolate.rs`

関数:

```rust
pub fn isolate_real_roots(p: &UniPolynomialQ, options: RootIsolationOptions) -> Result<Vec<RealRootRecord>, SolverError>;
```

疑似コード:

```rust
fn isolate_real_roots(p, options):
    p = squarefree_support(p)
    if options.method == Sturm:
        return isolate_real_roots_sturm(p)
    else if options.method == DescartesVincent:
        return isolate_real_roots_descartes(p)
    else:
        return deterministic_default_exact_isolation(p)
```

禁止:

```text
- floating-only root finding
- approximate roots without isolating intervals
```

### 26.3 `roots/decode.rs`

関数:

```rust
pub fn decode_candidates(target: VariableId, support: &UniPolynomialQ, roots: &[RealRootRecord]) -> Vec<TargetCandidate>;
```

疑似コード:

```rust
fn decode_candidates(target, support, roots):
    candidates = []
    for (i, root) in roots.enumerate():
        candidates.push(TargetCandidate{
            target,
            support_hash: support.hash,
            root_index: i,
            isolating_interval: root.interval,
            candidate_hash: hash(target, support.hash, i, root.interval),
        })
    return candidates
```

---

## 27. Real fiber classification

この節は exact image mode の設計である。candidate cover mode では実行しなくてよいが、API とデータ構造は最初から存在させる。

### 27.1 `fiber/exact_image.rs`

関数:

```rust
pub fn classify_real_target_image(
    system: &CompressedSystemQ,
    support: &UniPolynomialQ,
    candidates: &[TargetCandidate],
    ctx: &mut SolverContext,
) -> Result<FiberClassificationResult, SolverError>;
```

疑似コード:

```rust
fn classify_real_target_image(system, support, candidates, ctx):
    records = []
    for cand in candidates:
        fiber_problem = add_algebraic_target_condition(system, support, cand.root_index)
        semialgebraic = attach_slack_and_guard_semantics(fiber_problem)
        record = decide_real_fiber_nonempty(semialgebraic, ctx)?
        records.push(record)
    return FiberClassificationResult(records)
```

### 27.2 `fiber/hermite.rs`

関数:

```rust
pub fn hermite_real_root_count_for_fiber(input: HermiteFiberInput) -> Result<RealFiberCountCertificate, SolverError>;
```

用途:

```text
zero-dimensional fiber の実根数判定。
```

### 27.3 `fiber/thom.rs`

関数:

```rust
pub fn thom_sign_classify(input: ThomSignInput) -> Result<SignClassificationCertificate, SolverError>;
```

用途:

```text
algebraic target root 上で guard polynomial の符号を判定する。
```

### 27.4 `fiber/slack_semantics.rs`

関数:

```rust
pub fn apply_real_constraint_semantics(
    fiber: FiberProblem,
    semantics: &[RealConstraintEncoding],
) -> FiberProblemWithSemantics;

pub fn verify_slack_encoding_consistency(record: &FiberClassificationRecord) -> bool;
```

---

## 28. Result と diagnostics

### 28.1 `result/status.rs`

型:

```rust
pub enum SolverStatus { ... }

pub enum FailureKind {
    FiniteResourceFailure {
        stage: StageId,
        block_id: Option<BlockId>,
        matrix_rows: Option<usize>,
        matrix_cols: Option<usize>,
        matrix_density: Option<RationalQ>,
        quotient_rank_estimate: Option<usize>,
        coefficient_height_bits: Option<usize>,
        memory_bytes: Option<u64>,
    },
    AlgorithmicHardCase {
        stage: StageId,
        reason: AlgebraicReason,
        minimal_block_hash: Hash,
    },
    CertificateDesignGap {
        constructed_object_hash: Hash,
        missing_certificate_kind: String,
    },
    ImplementationBug {
        invariant_violated: String,
    },
}
```

### 28.2 `result/cost_trace.rs`

型:

```rust
pub struct GlobalCostTrace {
    pub total_variable_count: usize,
    pub total_relation_count: usize,
    pub total_monomial_count: usize,
    pub max_total_degree: usize,
    pub max_coefficient_height_bits: usize,
    pub max_block_width: usize,
    pub max_separator_width: usize,
    pub block_traces: Vec<ProjectionCostTrace>,
    pub composition_trace: CompositionCostTrace,
    pub verification_trace: VerificationCostTrace,
}

pub struct ProjectionCostTrace {
    pub block_id: BlockId,
    pub kernel_kind: KernelKind,
    pub local_variable_count: usize,
    pub exported_variable_count: usize,
    pub local_relation_count: usize,
    pub local_monomial_count: usize,
    pub estimated_quotient_rank: Option<usize>,
    pub matrix_rows: Option<usize>,
    pub matrix_cols: Option<usize>,
    pub matrix_density: Option<RationalQ>,
    pub coefficient_height_before_bits: usize,
    pub coefficient_height_after_bits: usize,
}
```

### 28.3 `result/output.rs`

関数:

```rust
pub fn finalize_success_result(input: FinalizeSuccessInput) -> TargetSolveResult;
pub fn finalize_failure_result(input: FinalizeFailureInput) -> TargetSolveResult;
```

---

## 29. Solver orchestrator

### 29.1 `solver/options.rs`

型:

```rust
pub struct SolverOptions {
    pub exact_image_mode: bool,
    pub max_memory_bytes: Option<u64>,
    pub max_matrix_rows: Option<usize>,
    pub max_matrix_cols: Option<usize>,
    pub max_coefficient_height_bits: Option<usize>,
    pub root_isolation_method: RootIsolationMethod,
    pub certificate_level: CertificateLevel,
}
```

### 29.2 `solver/pipeline.rs`

各 step 関数:

```rust
pub fn step_validate(problem: RationalTargetProblem, ctx: &mut SolverContext) -> Result<ValidatedProblem, SolverError>;
pub fn step_canonicalize(validated: ValidatedProblem, ctx: &mut SolverContext) -> Result<CanonicalSystemQ, SolverError>;
pub fn step_compress(canonical: CanonicalSystemQ, ctx: &mut SolverContext) -> Result<CompressedSystemQ, SolverError>;
pub fn step_build_graphs(compressed: &CompressedSystemQ, ctx: &mut SolverContext) -> Result<GraphBundle, SolverError>;
pub fn step_build_dag(graphs: &GraphBundle, compressed: &CompressedSystemQ, ctx: &mut SolverContext) -> Result<TargetProjectionDAG, SolverError>;
pub fn step_plan(dag: &TargetProjectionDAG, compressed: &CompressedSystemQ, ctx: &mut SolverContext) -> Result<Vec<KernelPlan>, SolverError>;
pub fn step_execute(dag: &TargetProjectionDAG, plans: &[KernelPlan], compressed: &CompressedSystemQ, ctx: &mut SolverContext) -> Result<Vec<ProjectionMessage>, SolverError>;
pub fn step_compose(dag: &TargetProjectionDAG, messages: Vec<ProjectionMessage>, ctx: &mut SolverContext) -> Result<ComposedProjection, SolverError>;
pub fn step_support(composed: ComposedProjection, target: VariableId, ctx: &mut SolverContext) -> Result<UniPolynomialQ, SolverError>;
pub fn step_roots(support: &UniPolynomialQ, ctx: &mut SolverContext) -> Result<(UniPolynomialQ, Vec<RealRootRecord>, Vec<TargetCandidate>), SolverError>;
```

### 29.3 `solver/orchestrator.rs`

関数:

```rust
pub fn solve_with_context(problem: RationalTargetProblem, ctx: SolverContext) -> Result<TargetSolveResult, SolverError>;
```

疑似コード:

```rust
fn solve_with_context(problem, mut ctx):
    validated = step_validate(problem, &mut ctx)?
    canonical = step_canonicalize(validated, &mut ctx)?
    compressed = step_compress(canonical, &mut ctx)?

    graphs = step_build_graphs(&compressed, &mut ctx)?
    dag = step_build_dag(&graphs, &compressed, &mut ctx)?

    plans = step_plan(&dag, &compressed, &mut ctx)?
    messages = step_execute(&dag, &plans, &compressed, &mut ctx)?

    for msg in messages:
        verify_projection_message(msg)?

    composed = step_compose(&dag, messages, &mut ctx)?
    support = step_support(composed, compressed.target, &mut ctx)?
    support_cert = verify_global_support(&support, &composed, &ctx)?

    sq = squarefree_support(&support)?
    roots = isolate_real_roots(&sq, ctx.options.root_isolation_method)?
    candidates = decode_candidates(compressed.target, &sq, &roots)

    if ctx.options.exact_image_mode:
        fiber = classify_real_target_image(&compressed, &sq, &candidates, &mut ctx)?
        status = status_from_fiber(fiber)
    else:
        fiber = None
        status = CertifiedCandidateCover

    cert = finalize_core_run_certificate(...)
    return finalize_success_result(...)
```

---

## 30. 計算量設計

### 30.1 変数

```text
n      = total variable count
m      = total equation count
d      = maximum total degree
s      = total monomial count
h      = maximum coefficient bit height
w      = maximum block variable width
τ      = maximum separator width
D_b    = local quotient/action rank of block b
M_b    = local resultant/Macaulay/relation-search matrix size of block b
N_b    = cost of target-action matvec in block b
δ      = degree of final support polynomial S(T)
κ      = certificate size
```

### 30.2 目標とする支配項

望ましい支配項:

```text
TotalCost ≈
    Σ_b LocalProjectionCost_b(d, s_b, h_b, D_b, M_b, N_b)
  + SeparatorCompositionCost(τ, δ)
  + SupportVerificationCost(κ)
  + RootIsolationCost(δ, h)
  + OptionalFiberClassificationCost
```

避ける設計:

```text
w, τ, D_b, M_b が小さいのに、主計算が total n に対して指数的になる設計。
```

### 30.3 代数コスト圧縮の記録

各 run は次を `GlobalCostTrace` に記録する。

```text
- total n,m,d,s,h
- max block width w
- max separator width τ
- each block local variable count
- each block local relation count
- each block estimated quotient rank
- each block matrix size
- each block coefficient height before/after
- final support degree δ
- certificate size κ
```

これは benchmark 設計ではなく、アルゴリズムの出力証明書に含める内部 trace である。

---

## 31. 非有限 target image の扱い

`I ∩ Q[T] = {0}` の場合、非零 support polynomial は存在しない。この場合、target image が有限でない可能性がある。

関数:

```rust
pub fn certify_nonfinite_target_image(composed: &ComposedProjection) -> Result<NonFiniteCertificate, SolverError>;
```

方法:

```text
- elimination ideal に target-only relation が存在しないことを示す。
- dimension / algebraic dependence certificate を使う。
- regular-chain / Groebner dimension information を使う。
```

証明できる場合:

```text
status = CertifiedNonFiniteTargetImage
```

証明できない場合:

```text
status = AlgorithmicHardCase
reason = no target-only relation and non-finiteness not certified
```

---

## 32. 幾何由来性の使い方

solver core は幾何名を見ない。しかし、幾何由来 system が持つ代数的 footprint は使う。

| 幾何由来の性質 | solver core が見る情報 |
|---|---|
| 補助点が多い | 変数数は多いが incidence graph が疎 |
| 構成が局所的 | 小さい separator、低 treewidth |
| 距離・垂直・円条件が多い | 低次数、特に二次式が多い |
| 中間量が多い | definitional / affine eliminability |
| 不等式・選択条件 | slack/guard encoding と semantic provenance |
| 交点構成 | 低次数 algebraic tower |
| target に無関係な補助構成 | target-independent component |

禁止:

```text
if relation came from circle: use circle solver
if variable role is point coordinate: use point solver
if problem is triangle: use triangle formula
```

許可:

```text
if incidence graph has small separator: decompose
if polynomial is affine in variable: eliminate
if support is sparse: use sparse resultant
if local quotient rank is small: use action Krylov
if explicit tower is detected algebraically: use NormTrace
```

---

## 33. 完了条件

この仕様の solver core が完成したと言える条件は次である。

```text
1. 任意の well-formed Q-polynomial target system が generic pipeline に入る。
2. geometry-name dispatch が存在しない。
3. problem-id / fixture-id / expected-answer dispatch が存在しない。
4. TargetProjectionDAG が全 valid input に対して作られる。
5. no useful separator の場合も one large block として generic target-direct kernel に送られる。
6. 各 block に deterministic KernelPlan が作られる。
7. UniversalTargetEliminationKernel が存在し、target/separator-only output を返す。
8. production path は full coordinate solution list を作らない。
9. production path は full coordinate RUR を作らない。
10. 成功時は S(T) が Q[T] に作られ、exact Q verification を通る。
11. root isolation は exact である。
12. decoded candidate は support hash と root index に bind される。
13. exact image mode は real fiber / guard / slack semantics を扱う。
14. 失敗は Unsupported ではなく、証拠付き status として返る。
15. cost trace に代数コスト圧縮に関する全 parameter が記録される。
16. hidden fallback が API 上不可能である。
17. narrow slice completion が API 上不可能である。
```

---

## 34. 最終まとめ

`R-GDTPK-Q / ACCTP-Q` は、幾何 handler の集合ではない。特定の三角形、円、接線、距離、面積だけを扱う solver でもない。入力は有理数係数多項式方程式系であり、solver core は幾何名を見ない。

この solver の研究上の強みは、全座標解を求めず、target に必要な代数情報だけを `TargetProjectionDAG` 上で局所的に射影・合成する点にある。高速化の根拠は、target 値数の単純な削減ではなく、全体消去の巨大な代数コストを、局所 block 幅、separator 幅、local quotient/action rank、sparse template size、target support degree に圧縮することである。

この仕様は、数学的 algorithm、実装 folder 構成、file 単位の関数配置、各関数の入力・処理・出力、証明書、失敗 status、禁止事項を固定したものである。実装者はこの文書だけを読めば、`geosolver-core` の solver core が何をし、何をしてはいけないかを判断できる。
