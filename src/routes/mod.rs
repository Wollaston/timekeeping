use axum::Router;

use crate::state::AppState;

pub mod clients;
pub mod invoices;
pub mod root;

pub fn routes() -> Router<AppState> {
    Router::new()
        .merge(root::routes())
        .nest("/clients", clients::routes())
        .merge(invoices::routes())
}
