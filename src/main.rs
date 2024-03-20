use axum::Router;
use timekeeping::app_router;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app_state = timekeeping::state::AppState::new()
        .await
        .expect("Could not create App State.");

    let app = Router::new().merge(app_router()).with_state(app_state);

    let listener = TcpListener::bind("localhost:8080").await.unwrap();

    axum::serve(listener, app).await.unwrap();

    Ok(())
}
