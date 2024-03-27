use axum::Router;

use crate::state::AppState;

pub mod clients;
pub mod invoices;
pub mod root;
pub mod timekeepers;
pub mod timekeeping;

pub fn routes() -> Router<AppState> {
    Router::new()
        .merge(root::routes())
        .nest("/clients", clients::routes())
        .nest("/timekeeping", timekeeping::routes())
        .nest("/timekeepers", timekeepers::routes())
        .merge(invoices::routes())
}
