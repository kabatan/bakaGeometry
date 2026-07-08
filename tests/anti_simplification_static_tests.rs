use std::{fs, path::Path};

#[test]
fn production_files_do_not_use_banned_names() {
    let banned = [
        "Unsupported",
        "expected",
        "fixture",
        "circle",
        "distance",
        "area",
        "incircle",
        "mixtilinear",
        "orthic",
        "coordinate_solution",
        "solve_all",
        "lex_param",
        "equations.len() != 2",
        "polynomials.len() != 2",
        "f64",
        "f32",
        "TODO",
        "panic!(\"unsupported\")",
        "v2_impl",
        "new_algo",
        "hack",
        "legacy",
        "temp_",
        "fallback_solver",
        "toy",
    ];

    for path in rust_files(Path::new("src")) {
        let contents = fs::read_to_string(&path).expect("production file should be readable");
        for pattern in banned {
            assert!(
                !contents.contains(pattern),
                "banned production pattern {pattern:?} found in {}",
                path.display()
            );
        }
    }
}

#[test]
fn crate_root_does_not_expose_route_modules() {
    let contents = fs::read_to_string("src/lib.rs").expect("crate root should be readable");
    for module in [
        "pub mod candidate_",
        "pub mod residual",
        "pub mod window",
        "pub mod linear_",
        "pub mod fallback_elimination",
        "pub use candidate_residual",
        "pub use candidates",
        "pub use compression",
        "pub use residual",
        "pub use window",
    ] {
        assert!(
            !contents.contains(module),
            "crate root exposes internal module pattern {module:?}"
        );
    }
}

#[test]
fn solver_production_does_not_fail_close_unbounded_proof_bounds() {
    let contents = production_portion("src/solver.rs");
    for pattern in ["resource:unbounded_proof_requires_bound"] {
        assert!(
            !contents.contains(pattern),
            "solver production reintroduced hidden bounded/unbounded shortcut {pattern:?}"
        );
    }
    assert!(contents.contains("GlobalSolveSchedule::from_limits"));
}

#[test]
fn resultant_route_does_not_use_total_degree_macaulay_support_helper() {
    let contents =
        fs::read_to_string("src/candidate_resultant.rs").expect("resultant source is readable");
    for pattern in ["monomials_up_to_degree", "enumerate_monomials"] {
        assert!(
            !contents.contains(pattern),
            "resultant route reintroduced total-degree Macaulay helper {pattern:?}"
        );
    }
    assert!(contents.contains("SparseResultantWitnessTrace"));
    assert!(contents.contains("all_subset_minkowski_sums"));
}

#[test]
fn tower_guard_replay_does_not_verify_against_empty_semantic_guards() {
    let contents = production_portion("src/candidate_tower.rs");
    assert!(contents.contains("verify_guard_certificate"));
    assert!(contents.contains("semantic_guards: system.semantic_guards.clone()"));
    assert!(!contents.contains("input_semantic_guard_records"));
    assert!(
        !contents.contains("semantic_guards: Vec::new()"),
        "tower guard replay must preserve InputSemanticNonzero provenance"
    );
}

fn rust_files(root: &Path) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    for entry in fs::read_dir(root).expect("src directory should exist") {
        let entry = entry.expect("src entry should be readable");
        let path = entry.path();
        if path.extension().is_some_and(|extension| extension == "rs") {
            files.push(path);
        }
    }
    files
}

fn production_portion(path: &str) -> String {
    let contents = fs::read_to_string(path).expect("production file should be readable");
    contents
        .split("\n#[cfg(test)]")
        .next()
        .unwrap_or(&contents)
        .to_string()
}
