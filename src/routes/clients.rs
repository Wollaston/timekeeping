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
        .route("/create", get(new_client_form).post(create_new_client))
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
    created_at: DateTime<Utc>,
    modified_at: DateTime<Utc>,
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
#[template(path = "clients/new_client_form.html")]
struct NewClientFormTemplate;

async fn new_client_form() -> impl IntoResponse {
    NewClientFormTemplate
}

#[derive(Template)]
#[template(path = "clients/created_client.html")]
struct CreatedClientTemplate;

async fn create_new_client(
    State(state): State<AppState>,
    Form(client): Form<ClientForCreate>,
) -> impl IntoResponse {
    // Run some queries
    let sql = "
        BEGIN TRANSACTION;

        LET $client = CREATE clients SET created_at = time::now(), modified_at = time::now(), client_details = $data;

        RETURN $client;

        COMMIT TRANSACTION;";

    let result = state.db.query(sql).bind(("data", client)).await.unwrap();

    dbg!(result);

    CreatedClientTemplate
}

async fn read(
    State(state): State<AppState>,
    id: Path<String>,
) -> Result<Json<Option<ClientForCreate>>, Error> {
    let client = state.db.select((CLIENTS, &*id)).await?;
    Ok(Json(client))
}

async fn update(
    State(state): State<AppState>,
    id: Path<String>,
    Json(client): Json<ClientForCreate>,
) -> Result<Json<Option<ClientForCreate>>, Error> {
    let client = state.db.update((CLIENTS, &*id)).content(client).await?;
    Ok(Json(client))
}

async fn delete(
    State(state): State<AppState>,
    id: Path<String>,
) -> Result<Json<Option<ClientForCreate>>, Error> {
    let client = state.db.delete((CLIENTS, &*id)).await?;
    Ok(Json(client))
}

async fn list(State(state): State<AppState>) -> Result<Json<Vec<Client>>, Error> {
    let clients: Vec<Client> = state.db.select(CLIENTS).await?;
    Ok(Json(clients))
}
