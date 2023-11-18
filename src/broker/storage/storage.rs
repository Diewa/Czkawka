#![allow(dead_code)] // Usuń to Dawidku jak chcesz widzieć warningi znowu

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

    pub fn create_topic(&mut self, topic_name: String) -> Result<String, Error> {
        if self.topic_exists(&topic_name) {
            // return Err("topic already exists");
        }

        let message_queue = MessageQueue::new();

        self.store.insert(topic_name.clone(), message_queue);
        Ok(topic_name)
    }

    pub fn add_message(&mut self, topic_name: String, message: Message) -> std::io::Result<i32> {
        if !self.topic_exists(&topic_name) {
            // Error topic doesn't exist
        }

        let queue = match self.store.get_mut(&topic_name) {
            Some(queue) => queue,
            None => return Ok(0), // how to return error?
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


/* Questions to Mimi
1. Co z konstruktorami dla Struct? Jak inicjalizować zmienne i jakie są wartości domyslne jeśli po prostu wywołam Struct::new()?
2. Jak stworzyć funkcję która zwraca pusty Result?
3. Czy do funkcji muszę przekazać self aby móc odwołać się do pól ze struktury? Tak jak to jest w Kopper.rs:38
4. jak dodać logger do aplikacji?
5. Jak zwrócić Error z konkretnym message?
 */