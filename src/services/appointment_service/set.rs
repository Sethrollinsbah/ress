use axum::{self, response::IntoResponse, Json, http::StatusCode, extract::State};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{Connection, Result, params};
use std::sync::Arc;
use crate::models::{
    Appointment,
    CreateAppointmentRequest,
    AppointmentResponse,
    AppState,
};

pub async fn set_appointment_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateAppointmentRequest> // Change to CreateAppointmentRequest
) -> impl axum::response::IntoResponse {
    let conn = &state.db_pool;

    match insert_appointment(conn, payload) {
        Ok(response) => (StatusCode::CREATED, Json(response)).into_response(), // Return AppointmentResponse
        Err(e) => {
            eprintln!("Error creating appointment: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create appointment").into_response()
        }
    }
}

pub fn insert_appointment(conn_pool: &Pool<SqliteConnectionManager>, payload: CreateAppointmentRequest) -> Result<AppointmentResponse, Box<dyn std::error::Error>> {
    // Use map_err to convert r2d2::Error to a boxed error
    let conn = conn_pool.get().map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
    
    let mut statement = conn.prepare(
        "INSERT INTO appointments (user_id) VALUES (?) RETURNING id",
    )?;
    statement.execute(params![payload.id])?; // Changed payload.id to payload.user_id based on context
    
    let mut statement = conn.prepare("SELECT last_insert_rowid()")?;
    let mut rows = statement.query([])?;
    if let Some(row) = rows.next()? {
        let id: i64 = row.get(0)?;
        Ok(AppointmentResponse { id: id.to_string() })
    } else {
        Err(Box::new(rusqlite::Error::QueryReturnedNoRows))
    }
}
