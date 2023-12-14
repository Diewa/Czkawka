use crate::api::receiver::MessagePayload;
use crate::topic::topic_service::TopicService;

// I want to use TopicService in this struct to verify if topic exists.
// Should I use reference to topic_service? Such approach requires defining lifetime annotation
pub struct PublisherService<'a> {
    topic_service: &'a TopicService
}

impl PublisherService {
    pub fn new(topic_service: & TopicService) -> Self {
        let publisher = PublisherService {
            topic_service: topic_service
        };
    }

    pub fn publish_message(topic_name: String, message: MessagePayload) {

    }
}