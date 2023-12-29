use std::sync::Arc;

use rocket::{Rocket, Build};
use rocket::fs::{FileServer, relative};

use crate::api;
use crate::topic::{
    publisher_service::PublisherService,
    topic_service::TopicService
};
use czkawka::kopper::Kopper;

const SEGMENT_SIZE: usize = 4000; 

pub fn router(config: &rocket::Config, db_folder: &str) -> Rocket<Build> {
    
    // DI management
    let kopper = Kopper::create(db_folder, SEGMENT_SIZE).expect("Can't create Kopper!");
    let topic_service = Arc::new(TopicService::new(kopper.clone()));
    let publisher_service = PublisherService::new(topic_service.clone(), kopper);

    rocket::custom(config)

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

        .mount("/", FileServer::from(relative!("src/broker/web")))
}