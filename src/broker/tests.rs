use rocket::Config;
use rocket::fs::{FileServer, relative};
use rocket::http::ContentType;
use crate::topic::topic_service::TopicService;
use crate::api;

fn get_client() -> rocket::local::blocking::Client
{
    let rocket = 
        rocket::build()

        // PUBLISH
        // .mount("/publish", routes![
        //     api::receiver::publish_message, /// TODO
        //     api::receiver::get_offset       /// TODO
        // ])

        // ADMIN
        .mount("/admin", routes![
            api::admin::get_topics,
            api::admin::create_topic,
        ])
        .manage(TopicService::new())

        .mount("/", FileServer::from(relative!("src/broker/web")));

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
        
        