#[test]
fn p12g_generality_stress_manifest_lists_required_cases() {
    let cases = [
        (
            "G1",
            "kernels::target_relation_search::tests::p12g_g1_projection_without_initial_target_only_relation_finds_support",
        ),
        (
            "G2",
            "kernels::action_krylov::tests::p12g_action_krylov_builds_non_target_only_quotient_action",
        ),
        (
            "G3",
            "preprocess::linear_affine::tests::p12g_g3_bilinear_structure_preprocesses_to_target_support_under_permutation",
        ),
        (
            "G4",
            "preprocess::linear_affine::tests::p12g_g4_quadratic_structure_preprocesses_to_target_support",
        ),
        (
            "G5",
            "preprocess::linear_affine::tests::p12g_g5_guarded_rational_affine_records_guard_and_preserves_target_support",
        ),
        (
            "G6",
            "compose::compose::tests::p12g_g6_multiseparator_composition_requires_child_message",
        ),
        (
            "G7",
            "kernels::norm_trace_projection::tests::p12g_g7_norm_trace_stress_covers_single_and_two_step_towers",
        ),
        (
            "G8",
            "compose::final_support::tests::p12g_candidate_cover_with_no_real_roots_is_empty_cover_not_hard_case",
        ),
    ];
    assert_eq!(cases.len(), 8);
    assert!(cases
        .iter()
        .all(|(_, test_name)| test_name.contains("p12g_")));

    let mut ids = cases
        .iter()
        .map(|(case_id, _)| *case_id)
        .collect::<Vec<_>>();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids, ["G1", "G2", "G3", "G4", "G5", "G6", "G7", "G8"]);
}
