//! Inbound HTTP adapter for the frontend (HTMX + page routes). Each handler
//! module maps HTTP routes to application services; see docs/system-design/base/dependency-seams.md.

use actix_files as fs;
use actix_web::web;

use crate::handlers;

/// Registers all frontend routes by delegating to per-feature handler modules.
pub fn configure(cfg: &mut web::ServiceConfig) {
    // Static files for logo and assets
    cfg.service(fs::Files::new("/static", "./lib/app/frontend/web/static").show_files_listing());

    handlers::home::configure(cfg);
    handlers::docs::configure(cfg);
    handlers::analytics::configure(cfg);
    handlers::mapping::configure(cfg);
    handlers::eval::configure(cfg);
}
