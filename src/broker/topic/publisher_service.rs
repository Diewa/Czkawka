use std::sync::Arc;
use serde::{Deserialize, Serialize};
use rocket::serde::json::serde_json;

use czkawka::kopper::Kopper;
use crate::topic::topic_service::TopicService;

#[derive(Serialize, Deserialize)]
pub struct MessagePayload {
    _metadata: Vec<String>,
    _message: String
}

// I want to use TopicService in this struct to verify if topic exists.
// Should I use reference to topic_service? Such approach requires defining lifetime annotation
pub struct PublisherService {
    topic_service: Arc<TopicService>,
    db: Kopper
}

impl PublisherService {
    pub fn new(topic_service: Arc<TopicService>, db: Kopper) -> Self {
        let publisher = PublisherService {
            topic_service: topic_service, db
        };
        publisher
    }

    pub fn publish_message(&self, topic_name: &str, message: MessagePayload) -> Result<(), std::io::Error> {
        if self.topic_service.topic_exists(topic_name)? {
            todo!()
        };

        let serialized_message = serde_json::to_string(&message).expect("Failed to serialize");
        self.db.write("someMessageQueueKeyOrSomething", &serialized_message)?;
        todo!()
    }
}