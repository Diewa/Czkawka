use std::sync::Arc;
use serde::{Deserialize, Serialize};

use kopperdb::kopper::*;
use crate::topic::topic_service::TopicService;

use rocket::serde::json::serde_json;

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

pub enum PublisherServiceError {
    Internal
}

impl PublisherService {
    pub fn new(topic_service: Arc<TopicService>, db: Kopper) -> Self {
        let publisher = PublisherService {
            topic_service: topic_service, db
        };
        publisher
    }

    pub fn publish_message(&self, topic_name: &str, _message: MessagePayload) -> Result<(), PublisherServiceError> {
        match self.topic_service.topic_exists(topic_name) {
            Ok(_) => (),
            Err(_) => return Err(PublisherServiceError::Internal),
        };

        let serialized_message = serde_json::to_string(&_message).expect("Failed to serialize");

        // TODO: Fix this with a publisherservice errors!!!
        self.db.write("someMessageQueueKeyOrSomething", &serialized_message).expect("Couldn't write to Kopper!");
        Ok(())
    }
}