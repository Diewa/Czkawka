use rocket::State;
use rocket::serde::json::Json;
use serde::Serialize;

use crate::copper::Copper;

#[derive(Serialize)]
pub struct ReadResponse {
    value: String,
    error: String
}

#[derive(Serialize)]
pub struct WriteResponse {
    error: String
}

// api
#[get("/read/<key>")]
pub fn read(key: String, state: &State<Copper>) -> Json<ReadResponse> {

    let mut response = ReadResponse { 
        value: String::from(""), 
        error: String::from("OK") 
    };

    match state.read(key) {
        Ok(value_option) => {
            match value_option {
                Some(value) => {
                    response.value = value
                },
                None => {
                    response.error = "No such thing in database".to_string()
                }
            }
        },
        Err(err) => {
            response.error = err.to_string()
        }
    };

    Json(response)
}

#[get("/write/<key>/<value>")]
pub fn write(key: String, value: String, state: &State<Copper>) -> Json<WriteResponse> {
    
    let result = match state.write(key, value) {
        Ok(_) => "OK".to_string(),
        Err(err) => format!("Error writing to database! ({})", err)
    };

    Json(WriteResponse { error: result.to_string() })
}

pub fn create_shared_state() -> Result<Copper, std::io::Error> {
    Copper::start("copper.db")
}
