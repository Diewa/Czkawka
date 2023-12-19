use rocket::http::ContentType;
use crate::router;

fn get_client() -> rocket::local::blocking::Client
{
    let rocket = router::router();
    rocket::local::blocking::Client::untracked(rocket).expect("Could not build the client")
}

#[test]
fn test_dupa()
{
    let client = get_client();

    let response = client.post("/admin/topics")
                                            .header(ContentType::Form)
                                            .body("name=asd&owner=fff")
                                            .dispatch();

    assert_eq!(response.status(), rocket::http::Status::Ok);
}
        
        