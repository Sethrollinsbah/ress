use serde::{Serialize, Deserialize};

#[derive(Debug)]
pub struct Appointment {
    pub id: String,
}

#[derive(Deserialize)]
pub struct CreateAppointmentRequest {
    pub id: String,
    // pub time: String,
    // pub date: String,
}

#[derive(Serialize)]
pub struct AppointmentResponse {
    pub id: String,
    // pub time: String,
    // pub status: String,
}
