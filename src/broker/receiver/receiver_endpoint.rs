
use crate::receiver::message::Message;

#[get("/publish/{}")]
pub fn publish_message(message: Message) -> &'static str {
    // to do: controller responsible for message publishing 
    "Hello, world!"
}

