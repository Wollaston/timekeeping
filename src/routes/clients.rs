use crate::error::Error;
use crate::state::AppState;
use askama::Template;
use askama_axum::IntoResponse;
use axum::extract::Path;
use axum::extract::State;
use axum::routing::post;
use axum::Form;
use axum::Json;
use axum::{routing::get, Router};
use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;
use surrealdb::sql::Thing;
use tracing::debug;

const CLIENTS: &str = "clients";

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(clients_handler))
        .route("/create", post(create))
        .route("/read/:id", get(read))
        .route("/update/:id", post(update))
        .route("/list", get(list))
        .route("/delete/:id", post(delete))
}

#[derive(Debug, Deserialize)]
struct Record {
    #[allow(dead_code)]
    id: Thing,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct Client {
    created_date: DateTime<Utc>,
    modified_date: DateTime<Utc>,
    client_details: ClientForCreate,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ClientForCreate {
    client_name: String,
    contact_name: String,
    email: String,
    phone: String,
    street_address: String,
    city: String,
    state: String,
    postal_code: String,
}

#[derive(Template)]
#[template(path = "clients/clients.html")]
struct ClientsTemplate;

async fn clients_handler() -> impl IntoResponse {
    debug!("{:<12} - clients_handler", "HANDLER");
    ClientsTemplate
}

#[derive(Template)]
#[template(path = "clients/created_client.html")]
struct CreatedClientTemplate;

pub async fn create(
    State(state): State<AppState>,
    Form(client): Form<ClientForCreate>,
) -> impl IntoResponse {
    let client: Vec<Record> = state
        .db
        .create(CLIENTS)
        .content(Client {
            created_date: Utc::now(),
            modified_date: Utc::now(),
            client_details: client,
        })
        .await.unwrap();
    dbg!(&client);
    CreatedClientTemplate
}

pub async fn read(
    State(state): State<AppState>,
    id: Path<String>,
) -> Result<Json<Option<ClientForCreate>>, Error> {
    let client = state.db.select((CLIENTS, &*id)).await?;
    Ok(Json(client))
}

pub async fn update(
    State(state): State<AppState>,
    id: Path<String>,
    Json(client): Json<ClientForCreate>,
) -> Result<Json<Option<ClientForCreate>>, Error> {
    let client = state.db.update((CLIENTS, &*id)).content(client).await?;
    Ok(Json(client))
}

pub async fn delete(
    State(state): State<AppState>,
    id: Path<String>,
) -> Result<Json<Option<ClientForCreate>>, Error> {
    let client = state.db.delete((CLIENTS, &*id)).await?;
    Ok(Json(client))
}

pub async fn list(State(state): State<AppState>) -> Result<Json<Vec<Client>>, Error> {
    let clients: Vec<Client> = state.db.select(CLIENTS).await?;
    Ok(Json(clients))
}