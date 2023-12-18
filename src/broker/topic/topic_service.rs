use std::{collections::HashMap, sync::{Arc, Mutex}};

use czkawka::kopper::*;
use serde::{Serialize, Deserialize};
use rocket::serde::json::serde_json;

const TOPICS_KEY: &str = "topics";

#[derive(Clone, Serialize, Deserialize)]
pub struct TopicEntry {
    pub name: String,
    pub owner: String
}

pub struct TopicService {

    // Arc< Mutex< InternalState > > - why?
    // It's a pattern in rust used to *share mutable access* to data.
    //
    // How to share mutable access? In rust:
    // 1. Variable can be referenced *immutably* multiple times simultaneously - a.k.a. "shared" reference
    // 2. Variable can be referenced *mutably* only once - a.k.a "exclusive" reference
    // so sharing a mutable reference should not be possibe. Right?
    // 
    // To safely allow it there exists a concept of *Interior Mutability*. 
    // Basically there are some structs that allow mutating variable via shared reference, which means
    // you don't need to pass it as "mut", but you can still use methods that modify internal state
    // (methods that behave like taking "&mut self" as argument).
    //
    // There are 3: Cell, RefCell, and ... Mutex!
    //
    // Why Mutex? Because when accessing state guarded by a Mutex the caller is guaranteed to be the only
    // one accessing the data, so restrictions provided by borrow checker (checking that mut == exclusive) 
    // are not needed anymore.
    //
    // Ok, so we have Mutex to do modifications, but why Arc< .. > ?
    // Arc stands for "Atomically Reference Counted" - it's a concept from C++ known as "shared pointer". 
    // 
    // Arc is a struct that acts as a reference to the object, but with a benefit of destroying the object
    // when there is nothing referencing it (no copy of Arc exists anymore). It's object's personal garbage collector. 
    //
    // Arc is used to reference data shared between threads. It doesn't provide safe access (that's what Mutex if for),
    // but ensures that object is deallocated when (and not before) the last thread knowing about it finishes.

    topics: Arc<Mutex< HashMap<String, TopicEntry> >>,

    // Database. KopperDB already has all the synchronizations mechanisms
    db: Kopper
}

impl TopicService {
    pub fn new(db: Kopper) -> Self {
        let ts = TopicService{ topics: Arc::default(), db };

        // // Mock some topics
        // ts.create_topic(TopicEntry { name: String::from("Dobry Topic"), owner: String::from("Dawid") });
        // ts.create_topic(TopicEntry { name: String::from("Chujowy Topic"), owner: String::from("Szatan") });
        // ts.create_topic(TopicEntry { name: String::from("Średni Topic"), owner: String::from("Michal") });
        ts
    }

    pub fn create_topic(&self, topic: TopicEntry) -> bool {

        // 1. Add a topic to the database
        self.db.write(&topic.name, &serde_json::to_string(&topic).expect("Failed to serialize"))
            .unwrap();

        // 2. Add the topic name (key) to list of topics
        match self.db.read(TOPICS_KEY) {
            Ok(value) => {
                match value {
                    Some(topic_list) => {
                        // Topic list exists! Deserialize it
                        let mut list: Vec<String> = serde_json::from_str(&topic_list).expect("Can't deserialize topic list");
                        
                        // Append to it
                        list.push(topic.name);

                        // And write it back
                        self.db.write(TOPICS_KEY, &serde_json::to_string(&list).expect("Failed to serialize")).unwrap();
                    },

                    // Topics list does not exist yet, let's add it
                    None => {
                        self.db.write(TOPICS_KEY, &serde_json::to_string(&vec![topic.name]).expect("Failed to serialize")).unwrap();
                    },
                }
                return true;
            },
            Err(error) => {
                return false;
            },
        }
    }

    pub fn get_topics(&self) -> Vec<TopicEntry> {
        self.topics
            .lock()
            .expect("Can't lock get_topics")
            .values()
            .cloned()
            .collect::<Vec<TopicEntry>>()
    }

    pub fn topic_exists(&self, topic_name: &String) -> bool {
        return self.topics.lock()
            .expect("Can't lock on topic_exists")
            .contains_key(topic_name);
    }
}

#[test]
fn test()
{
    let v: Vec<i32> = vec![1,2,3,4];

    let arc1 = Arc::new(v);

    let arc2 = arc1.clone();
    std::thread::spawn(move || {
        println!("{:?}", arc2);
    });

    std::thread::spawn(move || {
        println!("{:?}", arc1);
    });
}