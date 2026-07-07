#![allow(
    clippy::cloned_ref_to_slice_refs,
    clippy::items_after_test_module,
    clippy::large_enum_variant,
    clippy::manual_contains,
    clippy::manual_clamp,
    clippy::manual_is_multiple_of,
    clippy::manual_retain,
    clippy::needless_range_loop,
    clippy::filter_map_bool_then,
    clippy::iter_cloned_collect,
    clippy::module_inception,
    clippy::never_loop,
    clippy::question_mark,
    clippy::result_large_err,
    clippy::too_many_arguments,
    clippy::unnecessary_sort_by,
    clippy::unnecessary_map_or
)]

pub mod algebra;
pub mod api;
pub mod compose;
pub mod graph;
pub mod kernels;
pub mod planner;
pub mod preprocess;
pub mod problem;
pub mod result;
pub mod roots;
pub mod solver;
pub mod types;
pub mod verify;
