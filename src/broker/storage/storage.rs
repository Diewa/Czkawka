use std::{collections::HashMap, fmt::Error};
use std::collections::LinkedList;

struct Storage {
    store: HashMap<String, MessageQueue>,
}

struct Message {
    payload: String,
    metadata: String,
}

struct MessageQueue {
    queue: LinkedList<Message>, // Queue doesn't exists in rust
    offset: i32
}

impl Storage {

    // TODO: topic-related functions should be moved to separate struct e.g. TopicManager
    pub fn create_storage() -> Result<Self, Error> {
        Ok(Storage {
            store: HashMap::new()
        })
    }

    pub fn create_topic(&mut self, topic_name: String) -> Result<String, String> {
        if self.topic_exists(&topic_name) {
            return Err(format!("topic with name: {} already exists", topic_name))
        }

        let message_queue = MessageQueue::new();

        self.store.insert(topic_name.clone(), message_queue);
        Ok(topic_name)
    }

    pub fn add_message(&mut self, topic_name: String, message: Message) -> Result<i32, String> {
        if !self.topic_exists(&topic_name) {
            return Err(format!("Topic: {} doesn't exist", topic_name))
        }

        let queue = match self.store.get_mut(&topic_name) {
            Some(queue) => queue,
            None => return Err(format!("cannot find queue for topic: {}", topic_name))
        };

        queue.add_message(message);
        return Ok(queue.offset);
    }

    fn topic_exists(&self, topic_name: &String) -> bool {
        return self.store.contains_key(topic_name);
    }
}

impl MessageQueue {
    fn new() -> MessageQueue {
        MessageQueue {
            queue: LinkedList::new(),
            offset: 0
        }
    }

    fn add_message(&mut self, message: Message) {
        self.offset += 1;
        self.queue.push_back(message);
    }
}