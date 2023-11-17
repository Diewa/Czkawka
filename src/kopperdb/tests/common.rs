use std::ops::Deref;

use crate::api::*;

use rand::{Rng, distributions::Alphanumeric};
use rocket::Config;

const DB_PATH: &str = "testfiles";
const SEGMENT_SIZE: u64 = 1024;

pub struct TestClient {
    client: Option<rocket::local::blocking::Client>,
    seg_size: u64,
    test_path: String,
    folder_prefix: String
}

impl TestClient {
    pub fn new(folder_prefix: &str) -> Self {
        TestClient { seg_size: SEGMENT_SIZE, test_path: "".to_owned(), client: None, folder_prefix: folder_prefix.to_owned() }
    }

    pub fn set_seg_size(mut self, seg_size: u64) -> Self { 
        self.seg_size = seg_size; self 
    }

    pub fn build(mut self) -> Self {
        let config = Config {
            log_level: rocket::config::LogLevel::Off,
            ..Config::debug_default()
        };

        // Randomize test folder path unless we want to use the same
        if self.test_path.is_empty() {
            self.test_path = DB_PATH.to_owned() + "/" + &self.folder_prefix + "/" + &random_key_value_with_size(20).0;
        }

        println!("Creating database {}", self.test_path);
        
        self.client = Some(rocket::local::blocking::Client::untracked(
            rocket::custom(&config)
                .mount("/", routes![read_kopper, write_kopper, read_brass, write_brass])
                .manage(create_kopper(&self.test_path, self.seg_size).expect("Can't create kopper")) 
                .manage(create_brass(&self.test_path, self.seg_size).expect("Can't create brass")) 
                .manage(create_stats())
        ).expect("Could not build the client"));

        self
    }
}

impl Deref for TestClient {
    type Target = rocket::local::blocking::Client;

    fn deref(&self) -> &Self::Target {
        self.client.as_ref().expect("Derefed TestClient is None!")
    }
}

pub fn random_key_value_with_size(size: usize) -> (String, String) {

    let get_random_str = || {
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(size)
            .map(char::from)
            .collect()
    };
    (get_random_str(), get_random_str())
}

pub fn random_key_value() -> (String, String) {
    const LEN: usize = 10;
    random_key_value_with_size(LEN)
}