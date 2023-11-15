use std::io::Write;

use crate::kopper::Kopper;

use super::api::*;

use rand::{Rng, distributions::Alphanumeric};
use rocket::Config;

const DB_PATH: &str = "kopper_test";
const SEGMENT_SIZE: u64 = 1024;

struct TestClient {
    seg_size: u64,
    test_path: String
}

impl TestClient {
    fn new() -> Self {
        TestClient { seg_size: SEGMENT_SIZE, test_path: "".to_owned() }
    }

    fn set_seg_size(mut self, seg_size: u64) -> Self { self.seg_size = seg_size; self }  
    fn from_client(mut self, client: rocket::local::blocking::Client) -> Self { 
        self.test_path = client.rocket().state::<Kopper>().unwrap().path();
        drop(client);
        self 
    }

    fn build(self) -> rocket::local::blocking::Client {

        let config = Config {
            log_level: rocket::config::LogLevel::Off,
            ..Config::debug_default()
        };

        // Randomize test folder path unless we want to use the same
        let path;
        if self.test_path.is_empty() {
            path = DB_PATH.to_owned() + "/" + &random_key_value_with_size(20).0;
        }
        else {
            path = self.test_path;
        }

        println!("Creating database {}", path);
        
        let client = rocket::local::blocking::Client::untracked(
            rocket::custom(&config)
                .mount("/", routes![read_kopper, write_kopper])
                .manage(create_kopper(&path, self.seg_size).expect("Can't create opper")) 
                .manage(create_stats())
        )
        .expect("Could not build the client");
        client
    }
}

fn random_key_value_with_size(size: usize) -> (String, String) {

    let get_random_str = || {
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(size)
            .map(char::from)
            .collect()
    };
    (get_random_str(), get_random_str())
}

fn random_key_value() -> (String, String) {
    const LEN: usize = 10;
    random_key_value_with_size(LEN)
}

#[test]
fn test_write_read() {
    let client = TestClient::new().build();

    // Write
    let (key, value) = random_key_value();
    client.get(format!("/write/{}/{}", key, value)).dispatch();

    // Read
    let read_response = client.get(format!("/read/{}", key)).dispatch();

    assert_eq!(read_response.status(), rocket::http::Status::Ok);
    assert_eq!(read_response.into_string().unwrap(), format!("{{\"value\":\"{}\",\"error\":\"OK\"}}", value));
}

#[test]
fn database_recovers_after_dying() {
    let client = TestClient::new().build();

    let mut key_values = Vec::new();
    for i in 0..5 {
        key_values.push(random_key_value());
        client.get(format!("/write/{}/{}", key_values[i].0, key_values[i].1)).dispatch();
    }

    // All in-memory structure is dropped
    
    let client = TestClient::new().from_client(client).build();
    
    for i in key_values {
        let read_response = client.get(format!("/read/{}", i.0)).dispatch();
        assert_eq!(read_response.status(), rocket::http::Status::Ok);
        assert_eq!(read_response.into_string().unwrap(), format!("{{\"value\":\"{}\",\"error\":\"OK\"}}", i.1));
    }
}

#[test]
fn recover_all_files_from_folder() {
    // Create small segments
    let client = TestClient::new().set_seg_size(100).build();

    // Fill first file quickly
    for _ in 0..3 {
        let (key, value) = random_key_value_with_size(19);
        client.get(format!("/write/{}/{}", key, value)).dispatch();
    }

    // Meaningful value - should be in second file
    client.get("/write/meaningful/thing").dispatch();

    let read_response = client.get("/read/meaningful").dispatch();
    assert_eq!(read_response.into_string().unwrap(), "{\"value\":\"thing\",\"error\":\"OK\"}");
}

#[test]
fn database_does_not_grow_forever() {
    let client = TestClient::new().set_seg_size(14).build();

    // Send 10 identical requests
    let (key, value) = random_key_value_with_size(2);
    for _ in 0..10 {
        client.get(format!("/write/{}/{}", key, value)).dispatch();
    }

    // Verify that database is smaller than 10 x (key + value + 2)
    let all_entries_together_size = 10 * (2 + 2 + 2) / 2;
    let size = client.rocket().state::<Kopper>().unwrap().size();
    
    assert!(size < all_entries_together_size, "{} >= {}", size, all_entries_together_size);
}