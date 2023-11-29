use crate::receiver::storage::MessageStorage;

struct Receiver {
    storage: dyn MessageStorage
}

impl Receiver {
    fn create(storage: &impl MessageStorage) -> Result<Receiver, Err> {
        Ok(Receiver{
            storage: storage,
        })
    }

    fn publish_message(&self, payload: String, headers: Vec<String>) {
        self.storage.save_message(payload, headers)
    }
}