use std::ops::Deref;

use crate::api::*;

use rand::{Rng, distributions::Alphanumeric};
use rocket::Config;

const DB_PATH: &str = "testfiles";
const SEGMENT_SIZE: usize = 1024;

pub enum DBType {
    Kopper,
    Brass
}

pub struct TestClient {
    client: Option<rocket::local::blocking::Client>,
    seg_size: usize,
    test_path: String,
    db: DBType
}

impl TestClient {
    pub fn new(db: DBType) -> Self {
        TestClient { seg_size: SEGMENT_SIZE, test_path: "".to_owned(), client: None, db }
    }

    pub fn set_seg_size(mut self, seg_size: usize) -> Self { 
        self.seg_size = seg_size; self 
    }

    pub fn build(mut self) -> Self {
        let config = Config {
            log_level: rocket::config::LogLevel::Off,
            ..Config::debug_default()
        };

        // Randomize test folder path unless we want to use the same
        if self.test_path.is_empty() {
            let db_str = match self.db { DBType::Brass => "brass", DBType::Kopper => "kopper" };
            self.test_path = DB_PATH.to_owned() + "/" + db_str + "/" + &random_key_value_with_size(20).0;
        }

        println!("Creating database {}", self.test_path);
        
        let mut rocket = rocket::custom(&config);
        rocket = match self.db {
            DBType::Kopper => {
                rocket
                    .mount("/", routes![read_kopper, write_kopper])
                    .manage(create_kopper(&self.test_path, self.seg_size).expect("Can't create kopper")) 
                    .manage(create_stats())
            },
            DBType::Brass => {
                rocket
                    .mount("/", routes![read_brass, write_brass])
                    .manage(create_brass(&self.test_path, self.seg_size).expect("Can't create kopper")) 
                    .manage(create_stats())
            }
        };

        self.client = Some(
            rocket::local::blocking::Client::untracked(rocket).expect("Could not build the client")
        );
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