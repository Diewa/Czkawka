#![allow(unused)]

use rand::{distributions::Alphanumeric, Rng};
use rocket::http::ContentType;
use crate::router;

const DB_PATH: &str = "testfiles/broker";

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

pub fn get_client() -> rocket::local::blocking::Client {
    let config = rocket::Config {
        log_level: rocket::config::LogLevel::Off,
        ..rocket::Config::debug_default()
    };

    let db_path = DB_PATH.to_owned() + "/" + &random_key_value_with_size(20).0;

    println!("Creating database at {}", db_path);
    let rocket = router(&config, &db_path);

    rocket::local::blocking::Client::untracked(rocket).expect("Could not build the client")
}

// #[test]
fn test_create_topic()
{
    let client = get_client();

    let response = client.post("/admin/topics")
                                            .header(ContentType::Form)
                                            .body("name=asd&owner=fff")
                                            .dispatch();

    assert_eq!(response.status(), rocket::http::Status::Ok);

    let body = client.get("/admin/topics").dispatch().into_string().unwrap();
    assert!(body.contains("asd"));
    assert!(body.contains("fff"));
}

// #[test]
fn test_get_topic_info()
{
    let client = get_client();

    client.post("/admin/topics")
                                            .header(ContentType::Form)
                                            .body("name=toptopic&owner=mimi")
                                            .dispatch();

    // Add a subscriber
    let response = client.post("/admin/topic/toptopic/subscribe")
                                            .header(ContentType::Form)
                                            .body("name=new_sub&port=1234")
                                            .dispatch();

    assert_eq!(response.status(), rocket::http::Status::Ok);

    let response = client.get("/admin/topic/toptopic").dispatch();
    assert!(response.into_string().unwrap().contains("new_sub"));
}
        