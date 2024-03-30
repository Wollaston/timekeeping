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

const TIMEKEEPING: &str = "timekeeping";

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(timekeeping_handler))
        .route(
            "/create",
            get(new_timekeeping_form).post(create_new_timekeeping),
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
pub struct Timekeeping {
    id: Thing,
    timekeeper: Thing,
    client: Thing,
    notes: String,
    created_at: DateTime<Utc>,
    modified_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TimekeepingForCreate {
    timekeeper: String,
    client: String,
    notes: String,
}

async fn timekeeping_handler() -> impl IntoResponse {
    #[derive(Template)]
    #[template(path = "timekeeping/timekeeping.html")]
    struct TimekeepingTemplate;

    TimekeepingTemplate
}

async fn new_timekeeping_form(State(state): State<AppState>) -> impl IntoResponse {
    #[derive(Template)]
    #[template(path = "timekeeping/new_timekeeping_form.html")]
    struct NewTimekeepingFormTemplate {
        timekeeper_options: Vec<Option>,
        client_options: Vec<Option>,
    }

    #[derive(Deserialize)]
    struct Option {
        id: Thing,
        name: String,
    }

    // Run some queries
    let sql = "
    SELECT id, details.name AS name FROM timekeepers ORDER BY name ASC LIMIT 50;

    SELECT id, details.client_name AS name FROM clients ORDER BY name ASC LIMIT 50;
";
    let mut result = state.db.query(sql).await.unwrap();
    // Get the first result from the first query
    let timekeeper_options: Vec<Option> = result.take(0).unwrap();
    // Get all of the results from the second query
    let client_options: Vec<Option> = result.take(1).unwrap();

    NewTimekeepingFormTemplate {
        timekeeper_options,
        client_options,
    }
}

async fn create_new_timekeeping(
    State(state): State<AppState>,
    Form(timekeeping): Form<TimekeepingForCreate>,
) -> impl IntoResponse {
    #[derive(Template)]
    #[template(path = "timekeeping/created_timekeeping.html")]
    struct CreatedTimekeepingTemplate;

    // Run some queries
    let sql = "
        BEGIN TRANSACTION;

        LET $timekeeping = CREATE timekeeping SET created_at = time::now(), modified_at = time::now(), timekeeper = type::thing('timekeepers', $timekeeper), client = type::thing('clients', $client), notes = $notes;

        RETURN $timekeeping;

        COMMIT TRANSACTION;";

    let result = state
        .db
        .query(sql)
        .bind(("timekeeper", timekeeping.timekeeper))
        .bind(("client", timekeeping.client))
        .bind(("notes", timekeeping.notes))
        .await
        .unwrap();

    dbg!(result);

    CreatedTimekeepingTemplate
}

async fn read(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    #[derive(Template)]
    #[template(path = "timekeeping/timekeeping_detail.html")]
    struct TimekeepingDetailTemplate {
        data: TimekeepingDetail,
    }

    #[derive(Debug, Deserialize)]
    struct TimekeepingDetail {
        timekeeper_name: String,
        timekeeper_id: Thing,
        client_name: String,
        client_id: Thing,
        timekeeping_notes: String,
        timekeeping_id: Thing,
        created_at: DateTime<Utc>,
        modified_at: DateTime<Utc>,
    }

    let sql = "
        BEGIN TRANSACTION;

        LET $data = SELECT 
                        timekeeper.details.name AS timekeeper_name,
                        timekeeper.id AS timekeeper_id,
                        client.details.client_name AS client_name, 
                        client.id AS client_id,
                        notes AS timekeeping_notes,
                        created_at AS created_at,
                        modified_at AS modified_at,
                        id AS timekeeping_id
                    FROM type::thing('timekeeping', $id);

        RETURN $data;

        COMMIT TRANSACTION;";

    let data: Option<TimekeepingDetail> = state
        .db
        .query(sql)
        .bind(("id", id))
        .await
        .unwrap()
        .take(0)
        .expect("Error loading data.");

    dbg!(&data);

    let data = data.expect("Error loading data.");

    TimekeepingDetailTemplate { data }
}

async fn update(
    State(state): State<AppState>,
    id: Path<String>,
    Json(timekeeping): Json<TimekeepingForCreate>,
) -> Result<Json<Option<TimekeepingForCreate>>, Error> {
    let timekeeping = state
        .db
        .update((TIMEKEEPING, &*id))
        .content(timekeeping)
        .await?;
    Ok(Json(timekeeping))
}

async fn delete(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    #[derive(Template)]
    #[template(path = "timekeeping/timekeeping_deleted.html")]
    struct DeletedTimekeepingTemplate {
        id: String,
    }

    let result: Option<Timekeeping> = state.db.delete((TIMEKEEPING, id)).await.unwrap();

    let id = result.unwrap().id.id.to_string();

    DeletedTimekeepingTemplate { id }
}

async fn list(State(state): State<AppState>) -> impl IntoResponse {
    #[derive(Template)]
    #[template(path = "timekeeping/timekeeping_cards.html")]
    struct TimekeepingCardsTemplate {
        timekeepings: Vec<Timekeeping>,
    }

    #[derive(Template)]
    #[template(path = "timekeeping/timekeeping_card.html")]
    struct TimekeepingCardTemplate {
        timekeeping: Timekeeping,
    }

    #[derive(Template)]
    #[template(path = "timekeeping/no_timekeeping.html")]
    struct NoTimekeepingEntriesTemplate;

    let sql = "
    SELECT 
        id, 
        timekeeper,
        client,
        notes,
        created_at,
        modified_at
    FROM timekeeping;
";
    let mut result = state.db.query(sql).await.unwrap();

    let timekeepings: Vec<Timekeeping> = result.take(0).unwrap();

    match timekeepings.len() {
        0 => NoTimekeepingEntriesTemplate.into_response(),
        _ => TimekeepingCardsTemplate { timekeepings }.into_response(),
    }
}

mod filters {
    use chrono::{DateTime, Utc};
    pub fn format_dt(s: &DateTime<Utc>) -> ::askama::Result<String> {
        let dt = s.format("%Y-%m-%d %H:%M:%S").to_string();
        Ok(dt)
    }
}
