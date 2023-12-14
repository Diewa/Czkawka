use std::{collections::HashMap, sync::{Arc, Mutex}};

#[derive(Clone)]
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

    topics: Arc< Mutex< HashMap<String, TopicEntry> > >
}

impl TopicService {
    pub fn new() -> Self {
        let ts = TopicService{ topics: Arc::default() };

        // Mock some topics
        ts.create_topic(TopicEntry { name: String::from("Dobry Topic"), owner: String::from("Dawid") });
        ts.create_topic(TopicEntry { name: String::from("Chujowy Topic"), owner: String::from("Szatan") });
        ts.create_topic(TopicEntry { name: String::from("Średni Topic"), owner: String::from("Michal") });
        ts
    }

    pub fn create_topic(&self, topic: TopicEntry) -> bool {
        let mut locked_map = self.topics
            .lock() // Lock the Mutex, returns Result<HashMap, Error> - Result<T,Error> ? 
            .expect("Can't lock create_topic"); // Expect on the Result

        // We don't want to override existing keys
        if locked_map.contains_key(&topic.name) {
            return false;
        }

        locked_map.insert(topic.name.clone(), topic);
        true
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


// struct InnerArc<T>
// {
//     reference_counter: Mutex<u64>,
//     obj: T
// }

// struct Arc<T> {
//     inner: const * InnerArc<T>
// }

// impl Arc<T> {
//     new()
//     {
//         inner.reference_counter.lock() += 1;
//         return inner.obj;
//     }

//     ~destuctor()
//     {
//         if inner.reference_counter.lock() == 0
//         {
//             delete inner;
//         }
//         else 
//         {
//             inner.reference_counter.lock() -= 1;
//         }
//     }
// }

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