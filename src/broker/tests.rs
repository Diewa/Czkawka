use rocket::fs::{FileServer, relative};
use rocket::http::ContentType;
use crate::topic::topic_service::TopicService;
use crate::topic::publisher_service::PublisherService;
use crate::api;
use std::sync::Arc;
use czkawka::kopper::Kopper;

fn get_client() -> rocket::local::blocking::Client
{
    let kopper = Kopper::create("kopper_database", 4000).expect("Can't create Kopper!");
    let topic_service = Arc::new(TopicService::new(kopper.clone()));
    let publisher_service = PublisherService::new(topic_service.clone(), kopper);

    let rocket = 
            // DI management

    rocket::build()
        .configure(rocket::Config::figment().merge(("port", 8081)))

        // PUBLISH
        .mount("/publish", routes![
            api::receiver::publish_message,
            api::receiver::get_offset
        ])

        // ADMIN
        .mount("/admin", routes![
            api::admin::get_topics,
            api::admin::create_topic
        ])
        .manage(topic_service)
        .manage(publisher_service)

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
        
        