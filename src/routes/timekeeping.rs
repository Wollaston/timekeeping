use crate::state::AppState;
use askama::Template;
use askama_axum::IntoResponse;
use axum::routing::{get, Router};

pub fn routes() -> Router<AppState> {
    Router::new().route("/", get(timekeeping_handler))
}

#[derive(Template)]
#[template(path = "timekeeping/timekeeping.html")]
struct TimekeepingTemplate;

async fn timekeeping_handler() -> impl IntoResponse {
    TimekeepingTemplate
}
