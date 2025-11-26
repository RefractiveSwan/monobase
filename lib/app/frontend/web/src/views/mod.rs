pub mod components;
pub mod layout;
pub mod models;
pub mod pages;
pub mod partials;
pub mod styles;

// Re-export page render functions + fragments
pub use pages::{
    render_analytics_page, render_components_preview_page, render_dataset_admin_page,
    render_environment_page, render_eval_calibration_fragment, render_eval_compare_fragment,
    render_eval_jobs_fragment, render_eval_page, render_landing_page, render_log_fragment,
    render_mesh_page, render_observability_page, render_summary_fragment, render_workbench_page,
};
pub use partials::render_results_fragment;
