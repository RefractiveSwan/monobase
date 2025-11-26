use actix_web::{HttpResponse, Result, web};
use serde::Deserialize;

use crate::{
    handlers::home, state::AppState, vector::VectorMode,
    views::pages::workbench::render_vector_mode_fragment,
};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/settings/vector-mode").route(web::post().to(update_vector_mode)));
}

#[derive(Deserialize)]
pub struct VectorModeForm {
    pub mode: String,
}

async fn update_vector_mode(
    state: web::Data<AppState>,
    form: web::Form<VectorModeForm>,
) -> Result<HttpResponse> {
    let mode = match form.mode.as_str() {
        "enabled" => VectorMode::Enabled,
        "disabled" => VectorMode::Disabled,
        _ => state.vector_mode(),
    };
    state.set_vector_mode(mode);
    let config = home::build_vector_config(&state);
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(render_vector_mode_fragment(&config)))
}
