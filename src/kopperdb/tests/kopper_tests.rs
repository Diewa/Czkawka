use czkawka::kopper::Kopper;
use crate::tests::common::*;

#[test]
fn test_write_read() {
    let client = TestClient::new(DBType::Kopper).build();

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
    let client = TestClient::new(DBType::Kopper).build();

    let mut key_values = Vec::new();
    for i in 0..5 {
        key_values.push(random_key_value());
        client.get(format!("/write/{}/{}", key_values[i].0, key_values[i].1)).dispatch();
    }

    // All in-memory structure is dropped
    let client = client.build();
    
    for i in key_values {
        let read_response = client.get(format!("/read/{}", i.0)).dispatch();
        assert_eq!(read_response.status(), rocket::http::Status::Ok);
        assert_eq!(read_response.into_string().unwrap(), format!("{{\"value\":\"{}\",\"error\":\"OK\"}}", i.1));
    }
}

#[test]
fn recover_all_files_from_folder() {
    // Create small segments
    let client = TestClient::new(DBType::Kopper).set_seg_size(100).build();

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
    let client = TestClient::new(DBType::Kopper).set_seg_size(14).build();

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

#[test]
fn file_offset_is_set_correctly_after_recovery() {
    let client = TestClient::new(DBType::Kopper).set_seg_size(100).build();

    // Write to a file - offset is len(key + value) + 2
    client.get(format!("/write/some_key/222222")).dispatch();

    // Recreate memory part of database from files
    let client = client.build();
    
    // Write to a file again - offset should be recovered too, and correctly saved in in-memory table
    client.get(format!("/write/some_key/333333")).dispatch();

    let read_response = client.get("/read/some_key").dispatch();
    assert_eq!(read_response.into_string().unwrap(), "{\"value\":\"333333\",\"error\":\"OK\"}");
}