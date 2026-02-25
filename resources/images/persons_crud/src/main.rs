use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json};
use sqlx::{sqlite::SqlitePoolOptions, FromRow, SqlitePool};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;

#[derive(Clone)]
struct AppState {
    pool: SqlitePool,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
struct Person {
    id: i64,
    first_name: String,
    last_name: String,
    dni: String,
    address: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PersonRequest {
    first_name: String,
    last_name: String,
    dni: String,
    address: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Usa archivo en el directorio actual (debe existir)
    let database_url = "sqlite:persons.db";

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;

    inicializar_db(&pool).await?;

    let state = AppState { pool };

    let app = Router::new()
        .route("/persons", post(crear_person))
        .route("/persons", get(listar_persons))
        .route("/persons/:id", get(obtener_person))
        .route("/persons/:id", put(actualizar_person))
        .route("/persons/:id", delete(eliminar_person))
        .with_state(state)
        .layer(CorsLayer::permissive());

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));

    println!("Servidor en http://{}", addr);

    axum::serve(
        tokio::net::TcpListener::bind(addr).await?,
        app,
    )
    .await?;

    Ok(())
}

async fn inicializar_db(pool: &SqlitePool) -> anyhow::Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS person (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            first_name TEXT NOT NULL,
            last_name TEXT NOT NULL,
            dni TEXT NOT NULL UNIQUE,
            address TEXT NOT NULL
        );
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

async fn crear_person(
    State(state): State<AppState>,
    Json(payload): Json<PersonRequest>,
) -> impl IntoResponse {
    let result = sqlx::query(
        "INSERT INTO person (first_name, last_name, dni, address) VALUES (?, ?, ?, ?)",
    )
    .bind(&payload.first_name)
    .bind(&payload.last_name)
    .bind(&payload.dni)
    .bind(&payload.address)
    .execute(&state.pool)
    .await;

    match result {
        Ok(res) => (
            StatusCode::CREATED,
            Json(json!({ "id": res.last_insert_rowid() })),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}

async fn listar_persons(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, Person>(
        "SELECT id, first_name, last_name, dni, address FROM person",
    )
    .fetch_all(&state.pool)
    .await;

    match result {
        Ok(persons) => (
            StatusCode::OK,
            Json(serde_json::to_value(persons).unwrap()),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}

async fn obtener_person(
    Path(id): Path<i64>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, Person>(
        "SELECT id, first_name, last_name, dni, address FROM person WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await;

    match result {
        Ok(Some(person)) => (
            StatusCode::OK,
            Json(serde_json::to_value(person).unwrap()),
        ),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Person not found" })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}

async fn actualizar_person(
    Path(id): Path<i64>,
    State(state): State<AppState>,
    Json(payload): Json<PersonRequest>,
) -> impl IntoResponse {
    let result = sqlx::query(
        "UPDATE person SET first_name = ?, last_name = ?, dni = ?, address = ? WHERE id = ?",
    )
    .bind(&payload.first_name)
    .bind(&payload.last_name)
    .bind(&payload.dni)
    .bind(&payload.address)
    .bind(id)
    .execute(&state.pool)
    .await;

    match result {
        Ok(res) if res.rows_affected() > 0 => (
            StatusCode::OK,
            Json(json!({ "message": "Person updated" })),
        ),
        Ok(_) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Person not found" })),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}

async fn eliminar_person(
    Path(id): Path<i64>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let result = sqlx::query("DELETE FROM person WHERE id = ?")
        .bind(id)
        .execute(&state.pool)
        .await;

    match result {
        Ok(res) if res.rows_affected() > 0 => (
            StatusCode::OK,
            Json(json!({ "message": "Person deleted" })),
        ),
        Ok(_) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Person not found" })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}
