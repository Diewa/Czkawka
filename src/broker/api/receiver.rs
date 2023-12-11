use serde::Deserialize;
use rocket::serde::json::Json;

#[derive(Deserialize)]
pub struct MessagePayload {
    metadata: Vec<String>,
    message: String
}

#[post("/publish/<topic_name>", data = "<payload>")]
pub fn publish_message(topic_name: String, payload: Json<MessagePayload>) {
    // to do: controller responsible for message publishing 
   
}

/*
#[get("/hello/<name>")]
fn hello(name: &RawStr) -> String {
    format!("Hello, {}!", name.as_str())
}
*/

#[get("/offset")]
pub fn get_offset() -> &'static str {
    "offset"
}