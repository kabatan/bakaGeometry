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
        "temp",
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
