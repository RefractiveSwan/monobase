pub mod components;
pub mod layout;
pub mod models;
pub mod pages;
pub mod partials;
pub mod styles;

// Re-export page render functions + fragments
pub use pages::{
    render_analytics_page, render_components_preview_page, render_eval_fragment, render_eval_page,
    render_landing_page, render_workbench_page,
};
pub use partials::render_results_fragment;
