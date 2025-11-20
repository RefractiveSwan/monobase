use actix_web::{HttpResponse, Result, http::header, web};

use crate::state::AppState;

/// Registers `/docs` redirect.
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/docs").route(web::get().to(docs_redirect)));
}

pub async fn docs_redirect(state: web::Data<AppState>) -> Result<HttpResponse> {
    if let Some(url) = &state.config.docs_url {
        Ok(HttpResponse::Found()
            .append_header((header::LOCATION, url.as_str()))
            .finish())
    } else {
        Ok(HttpResponse::NotFound().body("Docs URL not configured for this environment."))
    }
}
