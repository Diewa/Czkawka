use kopperdb::kopper::*;
use serde::{Serialize, Deserialize};
use rocket::serde::json::serde_json;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum TopicServiceError {
    #[error("Topic {0} doesn't exist")]
    TopicNotFound(String),

    #[error(transparent)]
    Serde(#[from] serde_json::error::Error),

    #[error("Internal database error, check logs")]
    DatabaseError,
    // Note: There is no conversion from KopperError to DatabaseError on purpose. 
    // We want the topic service to handle database errors internally (by logging them)
    // and only bubble up info that the error happened - no details.
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TopicEntry {
    pub name: String,
    pub owner: String,
    pub subscribers: Vec<SubscriptionEntry>
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SubscriptionEntry {
    pub name: String,
    pub endpoint: String
}

// This is a tuple. Elements of tuple are accessed with indices: tuple.0
#[derive(Serialize, Deserialize)]
pub struct TopicList(pub Vec<TopicEntry>);

impl TopicList {
    fn to_json(self) -> serde_json::Result<String> {
        serde_json::to_string(&self.0)
    }

    fn from_json(str: &str) -> serde_json::Result<Self> {
        serde_json::from_str(&str)
    }

    fn key() -> &'static str {
        "topics"
    } 
}

pub struct TopicService {
    db: Kopper
}

impl TopicService {
    pub fn new(db: Kopper) -> Result<Self, TopicServiceError> {

        // Verify existence of the topic list entry
        if let Err(read_error) = db.read(TopicList::key()) {
            
            match read_error {
                
                KopperError::KeyDoesNotExist(_) => {
                    // Fresh database - let's create an empty entry
                    
                    let serialized_list = 
                        TopicList(vec![])
                            .to_json()
                            .expect("One can always serialize empty list");
    
                    if let Err(write_error) = db.write(TopicList::key(), &serialized_list) {
                        println!("Couldn't initialize a fresh topic list in db, error when writing: {write_error}");
                        return Err(TopicServiceError::DatabaseError);
                    }
                },
    
                _ => {
                    println!("Couldn't initialize a fresh topic list in db, error when reading: {read_error}");
                }
            }
        };
        
        Ok(TopicService{ db })
    }

    pub fn topic_exists(&self, topic_name: &str) -> Result<bool, TopicServiceError> {
        let topic_list = self.fetch_topic_list()?;

        Ok(topic_list.0
            .iter()
            .any(|x| x.name == topic_name)) // Match on names only
    }

    pub fn subscribe_topic(&self, topic_name: &str, subscription_entry: SubscriptionEntry) -> Result<(), TopicServiceError> {
        
        let mut topic_list =  self.fetch_topic_list()?;
        let topic = self.find_topic_in_list(topic_name, &mut topic_list)?;

        topic.subscribers.push(subscription_entry);

        self.save_topic_list(topic_list)?;
        Ok(())
    }

    pub fn get_topic(&self, topic_name: &str) -> Result<TopicEntry, TopicServiceError> {
        
        let mut topic_list =  self.fetch_topic_list()?;
        Ok(self.find_topic_in_list(topic_name, &mut topic_list)?.clone())
    }

    pub fn get_topics(&self) -> Result<TopicList, TopicServiceError> {
        self.fetch_topic_list()
    }

    pub fn create_topic(&self, topic: TopicEntry) -> Result<(), TopicServiceError> {

        // Fetch from DB
        let mut topic_list = self.fetch_topic_list()?;

        // Append our new topic to the list
        topic_list.0.push(topic);

        // Save to DB
        self.save_topic_list(topic_list)?;
        Ok(())
    }

    fn find_topic_in_list<'a>(&self, topic_name: &str, topic_list: &'a mut TopicList) -> Result<&'a mut TopicEntry, TopicServiceError> {
        let topic_entry_option = topic_list.0
            .iter_mut()
            .find(|x| x.name == topic_name);

        let topic_entry = topic_entry_option
            .ok_or_else(|| TopicServiceError::TopicNotFound(topic_name.to_owned()))?;

        Ok(topic_entry)
    }

    fn fetch_topic_list(&self) -> Result<TopicList, TopicServiceError> {
        
        let serialized_list = match self.db.read(TopicList::key()) {
            Ok(topic_list) => topic_list,

            Err(err) => {
                // Handle error by logging it and return 'redacted' error
                println!("Kopper error when reading {err}");
                
                return Err(TopicServiceError::DatabaseError);
            },
        };

        Ok(TopicList::from_json(&serialized_list)?)
    }

    fn save_topic_list(&self, topic_list: TopicList) -> Result<(), TopicServiceError> {
        
        let serialized_list = TopicList::to_json(topic_list)?;

        if let Err(err) = self.db.write(TopicList::key(), &serialized_list) {
            // Handle error by logging it and return 'redacted' error
            println!("Kopper error when writing {err}");

            return Err(TopicServiceError::DatabaseError);
        }

        Ok(())
    }
}



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

    // topics: Arc<Mutex< HashMap<String, TopicEntry> >>,

    // Database. KopperDB already has all the synchronizations mechanisms