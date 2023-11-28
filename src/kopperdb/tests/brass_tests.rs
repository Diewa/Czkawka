use crate::tests::common::*;

#[test]
fn test_write_read() {
    let client = TestClient::new(DBType::Brass).build();

    // Write
    let (key, value) = random_key_value();
    client.get(format!("/write/b/{}/{}", key, value)).dispatch();

    // Read
    let read_response = client.get(format!("/read/b/{}", key)).dispatch();

    assert_eq!(read_response.status(), rocket::http::Status::Ok);
    assert_eq!(read_response.into_string().unwrap(), format!("{{\"value\":\"{}\",\"error\":\"OK\"}}", value));
}

