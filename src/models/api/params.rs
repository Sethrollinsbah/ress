use serde::Deserialize;

#[derive(Deserialize)]
pub struct Params {
    pub filename: String,
}

#[derive(Deserialize)]
pub struct ParamsRunLighthouse {
    pub domain: String,
    pub email: String,
    pub name: String,
}
