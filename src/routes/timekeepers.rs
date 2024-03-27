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

const TIMEKEEPERS: &str = "timekeepers";

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(timekeepers_handler))
        .route(
            "/create",
            get(new_timekeeper_form).post(create_new_timekeeper),
        )
        .route("/:id", get(read).delete(delete))
        .route("/update/:id", post(update))
        .route("/list", get(list))
}

#[derive(Debug, Deserialize)]
struct Record {
    #[allow(dead_code)]
    id: Thing,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Timekeeper {
    id: Thing,
    created_at: DateTime<Utc>,
    modified_at: DateTime<Utc>,
    details: TimekeeperForCreate,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TimekeeperForCreate {
    name: String,
    title: String,
    email: String,
    phone: String,
    notes: String,
}

#[derive(Template)]
#[template(path = "timekeepers/timekeepers.html")]
struct TimekeepersTemplate;

async fn timekeepers_handler() -> impl IntoResponse {
    TimekeepersTemplate
}

#[derive(Template)]
#[template(path = "timekeepers/new_timekeeper_form.html")]
struct NewTimekeeperFormTemplate;

async fn new_timekeeper_form() -> impl IntoResponse {
    NewTimekeeperFormTemplate
}

#[derive(Template)]
#[template(path = "timekeepers/created_timekeeper.html")]
struct CreatedTimekeeperTemplate;

async fn create_new_timekeeper(
    State(state): State<AppState>,
    Form(timekeeper): Form<TimekeeperForCreate>,
) -> impl IntoResponse {
    // Run some queries
    let sql = "
        BEGIN TRANSACTION;

        LET $timekeeper = CREATE timekeepers SET created_at = time::now(), modified_at = time::now(), details = $data;

        RETURN $timekeeper;

        COMMIT TRANSACTION;";

    let result = state
        .db
        .query(sql)
        .bind(("data", timekeeper))
        .await
        .unwrap();

    dbg!(result);

    CreatedTimekeeperTemplate
}

#[derive(Template)]
#[template(path = "timekeepers/timekeeper_detail.html")]
struct TimekeeperDetailTemplate {
    timekeeper: Timekeeper,
}

async fn read(State(state): State<AppState>, id: Path<String>) -> impl IntoResponse {
    let timekeeper = state.db.select((TIMEKEEPERS, &*id)).await.unwrap().unwrap();
    TimekeeperDetailTemplate { timekeeper }
}

async fn update(
    State(state): State<AppState>,
    id: Path<String>,
    Json(timekeeper): Json<TimekeeperForCreate>,
) -> Result<Json<Option<TimekeeperForCreate>>, Error> {
    let timekeeper = state
        .db
        .update((TIMEKEEPERS, &*id))
        .content(timekeeper)
        .await?;
    Ok(Json(timekeeper))
}

#[derive(Template)]
#[template(path = "timekeepers/timekeeper_deleted.html")]
struct DeletedTimekeeperTemplate {
    name: String,
}

async fn delete(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    let result: Option<Timekeeper> = state.db.delete((TIMEKEEPERS, id)).await.unwrap();

    let name = result.unwrap().details.name;

    DeletedTimekeeperTemplate { name }
}

#[derive(Template)]
#[template(path = "timekeepers/timekeeper_cards.html")]
struct TimekeeperCardsTemplate {
    timekeepers: Vec<Timekeeper>,
}

#[derive(Template)]
#[template(path = "timekeepers/timekeeper_card.html")]
struct TimekeeperCardTemplate {
    timekeeper: Timekeeper,
}

#[derive(Template)]
#[template(path = "timekeepers/no_timekeepers.html")]
struct NotimekeepersTemplate;

async fn list(State(state): State<AppState>) -> impl IntoResponse {
    let timekeepers: Vec<Timekeeper> = state.db.select(TIMEKEEPERS).await.unwrap();
    match timekeepers.len() {
        0 => NotimekeepersTemplate.into_response(),
        _ => TimekeeperCardsTemplate { timekeepers }.into_response(),
    }
}

mod filters {
    use chrono::{DateTime, Utc};
    pub fn format_dt(s: &DateTime<Utc>) -> ::askama::Result<String> {
        let dt = s.format("%Y-%m-%d %H:%M:%S").to_string();
        Ok(dt)
    }
}
