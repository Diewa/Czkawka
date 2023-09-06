use super::api::*;

use rand::{Rng, distributions::Alphanumeric};
use rocket::Config;

const DB_PATH: &str = "kopper_test.db";

fn create_client() -> rocket::local::blocking::Client {

    let config = Config {
        log_level: rocket::config::LogLevel::Off,
        ..Config::debug_default()
    };
    
    let client = rocket::local::blocking::Client::untracked(
        rocket::custom(&config)
            .mount("/", routes![read, write])
            .manage(create_kopper(DB_PATH).expect("Can't create opper")) 
            .manage(create_stats())
    )
    .expect("Could not build the client");
    client
}

fn setup() -> rocket::local::blocking::Client {

    // Clean old database
    match std::fs::remove_file(DB_PATH) { _ => () }

    create_client()
}

fn random_key_value() -> (String, String) {
    const LEN: usize = 5;
    let get_random_str = || {
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(LEN)
            .map(char::from)
            .collect()
    };
    (get_random_str(), get_random_str())
}


#[test]
fn test_simple_write() {
    // Setup
    let client = setup();

    // Act
    let (key, value) = random_key_value();
    let response = client.get(format!("/write/{}/{}", key, value)).dispatch();
 
    // Test
    assert_eq!(response.status(), rocket::http::Status::Ok);
    assert_eq!(&response.into_string().unwrap(), "{\"error\":\"OK\"}");
}

#[test]
fn test_write_read() {
    let client = setup();

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
    let client = setup();

    let (key, value) = random_key_value();
    client.get(format!("/write/{}/{}", key, value)).dispatch();

    // All in-memory structure is dropped
    drop(client);
    
    let client = create_client();
    let read_response = client.get(format!("/read/{}", key)).dispatch();

    assert_eq!(read_response.status(), rocket::http::Status::Ok);
    assert_eq!(read_response.into_string().unwrap(), format!("{{\"value\":\"{}\",\"error\":\"OK\"}}", value));
}