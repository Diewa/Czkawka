use super::api::*;

fn client() -> rocket::local::blocking::Client {
    let client = rocket::local::blocking::Client::untracked(
        rocket::build()
            .mount("/", routes![read, write])
            .manage(create_shared_state()) 
    )
    .expect("Could not build the client");
    client
}

#[test]
fn test_read() {
    // Setup
    let client = client();

    // Act
    let response = client.get("/read/asd").dispatch();

    // Test
    assert_eq!(response.status(), rocket::http::Status::Ok);
    assert_eq!(&response.into_string().unwrap(), "{\"value\":\"Value for asd\",\"error\":\"OK\"}");
}

#[test]
fn test_write() {
    let client = client();
    let response = client.get("/write/asd/dsa").dispatch();

    assert_eq!(response.status(), rocket::http::Status::Ok);
    assert_eq!(&response.into_string().unwrap(), "{\"error\":\"OK\"}");
}