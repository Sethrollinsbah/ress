use rusqlite::{Connection, Result};
use crate::models::app::AppState;
use crate::models::{Person};

#[derive(Deserialize)]
pub struct CreateLeadRequest {
    pub name: String,
    pub email: String,
    // Add other lead fields as needed
}

#[derive(Serialize)]
pub struct LeadResponse {
    pub id: i64,
    pub name: String,
    pub email: String,
    // Add other lead fields as needed
}

pub async fn create_lead(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateLeadRequest>,
) -> impl IntoResponse {
    let conn = &state.db;

    match insert_lead(conn, payload) {
        Ok(lead) => (StatusCode::CREATED, Json(lead)).into_response(),
        Err(e) => {
            eprintln!("Error creating lead: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create lead").into_response()
        }
    }
}

pub fn insert_lead(conn: &Connection, payload: CreateLeadRequest) -> Result<LeadResponse> {
    let mut statement = conn.prepare(
        "INSERT INTO leads (name, email) VALUES (?, ?) RETURNING id, name, email",
    )?;
    statement.bind((1, payload.name.as_str()))?;
    statement.bind((2, payload.email.as_str()))?;

    if let sqlite::State::Row = statement.next()? {
        let id = statement.read::<i64, _>("id")?;
        let name = statement.read::<String, _>("name")?;
        let email = statement.read::<String, _>("email")?;

        Ok(LeadResponse { id, name, email })
    } else {
        Err(sqlite::Error {
            code: None,
            message: Some("Failed to retrieve inserted lead".to_string()),
        })
    }
}

fn sqlite_example() -> Result<()> {
    let conn = Connection::open_in_memory()?;

    conn.execute(
        "CREATE TABLE person (
            id    INTEGER PRIMARY KEY,
            name  TEXT NOT NULL,
            data  BLOB
        )",
        (), // empty list of parameters.
    )?;
    let me = Person {
        id: 0,
        name: "Steven".to_string(),
        data: None,
    };
    conn.execute(
        "INSERT INTO person (name, data) VALUES (?1, ?2)",
        (&me.name, &me.data),
    )?;

    let mut stmt = conn.prepare("SELECT id, name, data FROM person")?;
    let person_iter = stmt.query_map([], |row| {
        Ok(Person {
            id: row.get(0)?,
            name: row.get(1)?,
            data: row.get(2)?,
        })
    })?;

    for person in person_iter {
        println!("Found person {:?}", person.unwrap());
    }
    Ok(())
}


