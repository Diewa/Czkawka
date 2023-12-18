use std::sync::Arc;

use czkawka::kopper::Kopper;

use crate::api::receiver::MessagePayload;
use crate::topic::topic_service::TopicService;

// I want to use TopicService in this struct to verify if topic exists.
// Should I use reference to topic_service? Such approach requires defining lifetime annotation
pub struct PublisherService {
    topic_service: Arc<TopicService>,
    db: Kopper
}

impl PublisherService {
    pub fn new(topic_service: Arc<TopicService>, db: Kopper) -> Self {
        let publisher = PublisherService {
            topic_service: topic_service,
        };
        publisher
    }

    pub fn publish_message(&self, topic_name: String, message: MessagePayload) -> Result<(), &str> {
        if !self.topic_service.topic_exists(&topic_name) {
            return Err("Topic doesn't exist!");
        }

        Ok(())
    }
}