use czkawka::kopper::*;
use serde::{Serialize, Deserialize};
use rocket::serde::json::serde_json;


pub enum TopicServiceError {
    TopicNotFound(String),
    Internal
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
    // Consume the list
    fn to_json(self) -> String {
        serde_json::to_string(&self.0)
            .expect("Failed to serialize topic list")
    }

    // Create new list
    fn from_json(str: String) -> Self {
        serde_json::from_str(&str)
            .expect("Can't deserialize topic list")
    }

    pub fn iter(&self) -> std::slice::Iter<TopicEntry> {
        self.0.iter()
    }

    fn key() -> &'static str {
        "topics"
    } 
}

pub struct TopicService {
    db: Kopper
}

impl TopicService {
    pub fn new(db: Kopper) -> Self {
        TopicService{ db }
    }

    pub fn create_topic(&self, topic: TopicEntry) -> Result<usize, TopicServiceError> {

        let mut entry_list = self.fetch_topic_list()?;

        // Append our new topic to the list
        entry_list.0.push(topic);

        // Write the list back to db
        let serialized_list = TopicList::to_json(entry_list);

        let db_size = match self.db.write(TopicList::key(), &serialized_list) {
            Ok(x) => x,

            // TODO: Fix with new errors!
            Err(err) => return Err(TopicServiceError::Internal),
        };
        Ok(db_size)
    }

    pub fn get_topics(&self) -> Result<TopicList, TopicServiceError> {
        self.fetch_topic_list()
    }

    pub fn topic_exists(&self, topic_name: &str) -> Result<bool, TopicServiceError> {
        let topic_list = self.fetch_topic_list()?;

        Ok(topic_list.0
            .iter()
            .any(|x| x.name == topic_name)) // Match on names only
    }

    fn find_topic_in_list<'a>(&self, topic_name: &str, topic_list: &'a TopicList) -> Result<&'a TopicEntry, TopicServiceError> {
        // czemu tu uzywamy topic_list.0 skoro metoda iter() jest zaimplementowa w strukturze i robi dokładnie to samo?
        let option = topic_list
            .iter()
            .find(|x| x.name == topic_name);

        match option {
            Some(entry) => {
                return Ok(entry)
            }

            None => Err(TopicServiceError::TopicNotFound(format!("Topic with name {} not found", topic_name)))
        }
    }

    pub fn subscribe_topic(&self, topic_name: &str, subscription_entry: SubscriptionEntry) -> Result<(), TopicServiceError> {
        let mut entry_list = self.fetch_topic_list()?;
        let mut topic = self.find_topic_in_list(topic_name, &entry_list)?;

        //topic.subscribers.push(subscription_entry);

        let serialized_list = TopicList::to_json(entry_list);

        //self.db.write(TopicList::key(), &serialized_list)?;
        Ok(())
    }


    // todo: add cache
    fn fetch_topic_list(&self) -> Result<TopicList, TopicServiceError> {
        
        match self.db.read(TopicList::key()) {
            Ok(topic_list) => {
                // Found the entry, let's deserialize it
                Ok(TopicList::from_json(topic_list))
            },
            Err(err) => {
                println!("Can't fetch topic list due to: {}", err);
                
                // TODO: Fix this with dedicated topicservice error!!!
                Err(TopicServiceError::Internal)
            },
        }
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