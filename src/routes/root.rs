use askama::Template;
use askama_axum::IntoResponse;
use axum::{routing::get, Router};
use tracing::debug;

use crate::state::AppState;

#[derive(Template)]
#[template(path = "root/root.html")]
struct RootTemplate;

pub fn routes() -> Router<AppState> {
    Router::new().route("/", get(root_handler))
}

async fn root_handler() -> impl IntoResponse {
    debug!("{:<12} - root_handler", "HANDLER");
    RootTemplate
}
