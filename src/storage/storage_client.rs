pub struct StorageClient {
    db: Kopper
}

impl StorageClient {
    pub fn new(db: Kopper) -> Self {
        // do we have to create empty values?
    }

    // one big struct with lot of functions like: get_topic, get_topics, get_subscriptions etc.
    // or create dedicated structures for each EntryType e.g. TopicStorage

}


pub struct TopicStorage {
    db: KVNamespaceClient<Kopper>,
    prefix: String
}


impl TopicStorage {

    pub fn create() -> Self {
        TopicStorage {}
    }

    // how to set static value like: priv static topic_prefix = "topic_"

    pub fn read_topic(&self, topic_name: &str) {
        self.client.read(namespace, key)
    }


    fn conduct_index(&self, topic_name: &str) {
        "topic_" + topic_name
    }
}

trait KVStore {
    fn read();
    fn write();
}

impl KVStore for Kopper {
    ///...
}

pub struct KVNamespaceClient<Db: KVStore> {
    db: Db,
    registry: HashMap<String, String>
}

impl<Db: KVStore> KVNamespaceClient<Db> {

    pub fn from_db(db: Db) -> Self {
        KVNamespaceClient { db, registry: HashMap::default() }
    }

    pub fn register_namespace(&self, namespace: &str, prefix: &str) {
        if registry.contains_key(namespace) {
            panic!("O kurfa")
        }

        self.registry.insert(namespace, prefix);
    }

    pub fn write_to(&self, key: &str, value: &str, namespace: &str) -> Result<(), Error> {
        self.write(... key.to_owned(), value);
    }

    pub fn read_from(&self, key: &str, namespace: &str) -> String {

    }
}



