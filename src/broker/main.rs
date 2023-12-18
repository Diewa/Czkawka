use std::sync::Arc;

#[macro_use] extern crate rocket;
use rocket::fs::{FileServer, relative};

use crate::topic::publisher_service::PublisherService;
use crate::topic::topic_service::TopicService;
use czkawka::kopper::Kopper;

// Mods
mod api;
mod storage;
mod topic;

#[cfg(test)]
mod tests;

const KOPPERDB_FOLDER: &str = "kopper_database";
const SEGMENT_SIZE: usize = 4000; 

#[launch]
fn rocket() -> _ {

    // DI management
    let kopper = Kopper::create(KOPPERDB_FOLDER, SEGMENT_SIZE).expect("Can't create Kopper!");
    let topic_service = Arc::new(TopicService::new(kopper.clone()));
    let publisher_service = PublisherService::new(topic_service.clone(), kopper);

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

        .mount("/", FileServer::from(relative!("src/broker/web")))
}