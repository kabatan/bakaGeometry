fn function_body<'a>(source: &'a str, name: &str) -> &'a str {
    let needle = format!("fn {name}");
    let start = source
        .find(&needle)
        .unwrap_or_else(|| panic!("function {name} not found"));
    let brace_start = source[start..]
        .find('{')
        .map(|idx| start + idx)
        .unwrap_or_else(|| panic!("function {name} has no body"));
    let bytes = source.as_bytes();
    let mut depth = 0usize;
    for idx in brace_start..bytes.len() {
        match bytes[idx] {
            b'{' => depth = depth.saturating_add(1),
            b'}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return &source[brace_start..=idx];
                }
            }
            _ => {}
        }
    }
    panic!("function {name} body is unterminated");
}

fn assert_body_absent(source: &str, name: &str, forbidden: &[&str]) {
    let body = function_body(source, name);
    for token in forbidden {
        assert!(
            !body.contains(token),
            "{name} must not contain plan-time output construction token {token}"
        );
    }
}

fn assert_body_contains(source: &str, name: &str, required: &[&str]) {
    let body = function_body(source, name);
    for token in required {
        assert!(
            body.contains(token),
            "{name} must contain execute-time certification token {token}"
        );
    }
}

#[test]
fn fcr_plan_sparse_resultant_does_not_construct_output_relation() {
    let source = include_str!("../src/kernels/sparse_resultant.rs");
    let forbidden = ["build_sparse_resultant_trace", "compute_resultant_relation"];
    assert_body_absent(source, "plan_sparse_resultant_with_messages", &forbidden);
    assert_body_absent(source, "probe_sparse_resultant_plan", &forbidden);
}

#[test]
fn fcr_plan_specialization_interpolation_does_not_run_inner_kernel() {
    let source = include_str!("../src/kernels/specialization_interpolation.rs");
    let forbidden = [
        "build_specialization_interpolation_trace",
        "execute_inner_target_only_kernel",
        "execute_target_relation_search",
        "eliminate_to_keep_variables",
    ];
    assert_body_absent(
        source,
        "plan_specialization_interpolation_with_messages",
        &forbidden,
    );
    assert_body_absent(
        source,
        "probe_specialization_interpolation_plan",
        &forbidden,
    );
}

#[test]
fn fcr_plan_regular_chain_does_not_project_chain() {
    let source = include_str!("../src/kernels/regular_chain_projection.rs");
    let forbidden = [
        "build_regular_chain_trace",
        "local_regular_chain_decomposition",
        "project_chain_to_variables",
        "combine_chain_projections",
    ];
    assert_body_absent(source, "plan_regular_chain_projection", &forbidden);
    assert_body_absent(source, "probe_regular_chain_plan", &forbidden);
}

#[test]
fn fcr_plan_norm_trace_does_not_construct_norm_relation() {
    let source = include_str!("../src/kernels/norm_trace_projection.rs");
    let forbidden = ["build_norm_trace_trace", "norm_relation_for_tower_plan"];
    assert_body_absent(source, "plan_norm_trace_projection", &forbidden);
    assert_body_absent(source, "probe_norm_trace_plan", &forbidden);
}

#[test]
fn fcr_plan_target_action_does_not_construct_final_support() {
    let source = include_str!("../src/kernels/action_krylov.rs");
    let forbidden = [
        "build_target_action_krylov_trace",
        "build_production_target_relevant_quotient_handle",
        "block_krylov_sequence",
        "recover_recurrence",
        "certify_krylov_coverage",
    ];
    assert_body_absent(
        source,
        "plan_target_action_krylov_with_messages",
        &forbidden,
    );
    assert_body_absent(source, "probe_target_action_krylov_plan", &forbidden);
}

#[test]
fn fcr_plan_universal_elimination_does_not_run_inner_execution() {
    let source = include_str!("../src/kernels/universal_elimination.rs");
    let forbidden = [
        "compute_resultant_relation",
        "execute_target_relation_search",
        "eliminate_to_keep_variables",
        "local_regular_chain_decomposition",
        "project_chain_to_variables",
        "norm_relation_for_tower_plan",
    ];
    assert_body_absent(source, "plan_universal_elimination", &forbidden);
    assert_body_absent(
        source,
        "plan_universal_elimination_with_messages",
        &forbidden,
    );
}

#[test]
fn fcr_execute_recomputes_and_certifies_declared_plan_outputs() {
    assert_body_contains(
        include_str!("../src/kernels/sparse_resultant.rs"),
        "execute_sparse_resultant",
        &[
            "build_sparse_resultant_trace",
            "sparse_resultant_certificate_hash",
            "KernelCertificate::from_execution_plan_with_payload",
        ],
    );
    assert_body_contains(
        include_str!("../src/kernels/specialization_interpolation.rs"),
        "execute_specialization_interpolation",
        &[
            "build_specialization_interpolation_trace",
            "verify_interpolated_relation_by_elimination",
            "KernelCertificate::from_execution_plan_with_payload",
        ],
    );
    assert_body_contains(
        include_str!("../src/kernels/regular_chain_projection.rs"),
        "execute_regular_chain_projection",
        &[
            "build_regular_chain_trace",
            "regular_chain_certificate_hash",
            "KernelCertificate::from_execution_plan_with_payload",
        ],
    );
    assert_body_contains(
        include_str!("../src/kernels/norm_trace_projection.rs"),
        "execute_norm_trace_projection",
        &[
            "build_norm_trace_trace",
            "norm_trace_certificate_hash",
            "KernelCertificate::from_execution_plan_with_payload",
        ],
    );
    assert_body_contains(
        include_str!("../src/kernels/action_krylov.rs"),
        "execute_target_action_krylov",
        &[
            "build_target_action_krylov_trace",
            "target_action_krylov_certificate_hash",
            "KernelCertificate::from_execution_plan_with_payload",
        ],
    );
}
