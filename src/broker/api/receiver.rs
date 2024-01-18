use std::sync::Arc;

use rocket::{serde::json::Json, State};

use crate::topic::publisher_service::*;

#[post("/publish/<topic_name>", data = "<payload>")]
pub fn publish_message(
    topic_name: &str, 
    payload: Json<MessagePayload>, 
    publisher_service: &State<Arc<PublisherService>>) {
    // to do: controller responsible for message publishing 

    let _ = publisher_service.publish_message(topic_name, payload.0);
}

#[get("/offset")]
pub fn get_offset() -> &'static str {
    "offset"
}