mod error;
mod routes;
pub mod state;

use axum::Router;
use state::AppState;
use tower_http::services::ServeDir;

pub fn app_router() -> Router<AppState> {
    Router::new().merge(routes::routes())
        .nest_service("/public", ServeDir::new("public"))
}
