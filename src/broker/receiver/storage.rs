pub trait MessageStorage {
    fn save_message(&self, payload: String, metadata: Vec<String>); // TBD
}