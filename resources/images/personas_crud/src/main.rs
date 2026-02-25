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
struct Persona {
    id: i64,
    nombres: String,
    apellidos: String,
    dni: String,
    direccion: String,
}

#[derive(Debug, Deserialize)]
struct PersonaRequest {
    nombres: String,
    apellidos: String,
    dni: String,
    direccion: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Usa archivo en el directorio actual (debe existir)
    let database_url = "sqlite:personas.db";

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;

    inicializar_db(&pool).await?;

    let state = AppState { pool };

    let app = Router::new()
        .route("/personas", post(crear_persona))
        .route("/personas", get(listar_personas))
        .route("/personas/:id", get(obtener_persona))
        .route("/personas/:id", put(actualizar_persona))
        .route("/personas/:id", delete(eliminar_persona))
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
        CREATE TABLE IF NOT EXISTS persona (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            nombres TEXT NOT NULL,
            apellidos TEXT NOT NULL,
            dni TEXT NOT NULL UNIQUE,
            direccion TEXT NOT NULL
        );
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

async fn crear_persona(
    State(state): State<AppState>,
    Json(payload): Json<PersonaRequest>,
) -> impl IntoResponse {
    let result = sqlx::query(
        "INSERT INTO persona (nombres, apellidos, dni, direccion) VALUES (?, ?, ?, ?)",
    )
    .bind(&payload.nombres)
    .bind(&payload.apellidos)
    .bind(&payload.dni)
    .bind(&payload.direccion)
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

async fn listar_personas(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, Persona>(
        "SELECT id, nombres, apellidos, dni, direccion FROM persona",
    )
    .fetch_all(&state.pool)
    .await;

    match result {
        Ok(personas) => (
            StatusCode::OK,
            Json(serde_json::to_value(personas).unwrap()),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}

async fn obtener_persona(
    Path(id): Path<i64>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, Persona>(
        "SELECT id, nombres, apellidos, dni, direccion FROM persona WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await;

    match result {
        Ok(Some(persona)) => (
            StatusCode::OK,
            Json(serde_json::to_value(persona).unwrap()),
        ),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Persona no encontrada" })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}

async fn actualizar_persona(
    Path(id): Path<i64>,
    State(state): State<AppState>,
    Json(payload): Json<PersonaRequest>,
) -> impl IntoResponse {
    let result = sqlx::query(
        "UPDATE persona SET nombres = ?, apellidos = ?, dni = ?, direccion = ? WHERE id = ?",
    )
    .bind(&payload.nombres)
    .bind(&payload.apellidos)
    .bind(&payload.dni)
    .bind(&payload.direccion)
    .bind(id)
    .execute(&state.pool)
    .await;

    match result {
        Ok(res) if res.rows_affected() > 0 => (
            StatusCode::OK,
            Json(json!({ "message": "Persona actualizada" })),
        ),
        Ok(_) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Persona no encontrada" })),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}

async fn eliminar_persona(
    Path(id): Path<i64>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let result = sqlx::query("DELETE FROM persona WHERE id = ?")
        .bind(id)
        .execute(&state.pool)
        .await;

    match result {
        Ok(res) if res.rows_affected() > 0 => (
            StatusCode::OK,
            Json(json!({ "message": "Persona eliminada" })),
        ),
        Ok(_) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Persona no encontrada" })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}
