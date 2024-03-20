use askama::Template;
use askama_axum::IntoResponse;
use axum::{routing::get, Router};
use tracing::debug;

use crate::state::AppState;

#[derive(Template)]
#[template(path = "invoices/invoices.html")]
struct InvoicesTemplate;

pub fn routes() -> Router<AppState> {
    Router::new().route("/invoices", get(invoices_handler))
}

async fn invoices_handler() -> impl IntoResponse {
    debug!("{:<12} - invoices_handler", "HANDLER");
    InvoicesTemplate
}
