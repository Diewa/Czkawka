pub trait MessageStorage {
    fn save_message(payload: String, metadata: Vec<String>); // TBD
}