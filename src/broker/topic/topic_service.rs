use std::collections::HashMap;

pub struct TopicService {
    topics: HashMap<String, TopicEntry>
}

pub struct TopicEntry {
    name: String,
    owner: String
}

impl TopicService {
    pub fn new() -> Self {
        TopicService{}
    }

    pub fn create_topic(&mut self, name: String, topic: TopicEntry) {
        self.topics.insert(name, topic);
    }

    pub fn get_topics(&self) -> Vec<TopicEntry> {
        self.topics.values().cloned().collect::<Vec<TopicEntry>>()
    }
}