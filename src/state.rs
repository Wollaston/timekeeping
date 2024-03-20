use anyhow::Result;
use axum::extract::FromRef;
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;

#[derive(Clone, FromRef)]
pub struct AppState {
    pub db: Surreal<Client>,
}

impl AppState {
    pub async fn new() -> Result<AppState> {
        let db = Surreal::new::<Ws>("127.0.0.1:9000").await?;

        db.signin(Root {
            username: "root",
            password: "root",
        })
        .await?;

        db.use_ns("test").use_db("test").await?;

        let app_state = AppState { db };

        Ok(app_state)
    }
}
