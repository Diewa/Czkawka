use std::sync::Arc;

use rocket::fs::{FileServer, relative};
use crate::topic::publisher_service::PublisherService;
use crate::topic::topic_service::TopicService;

#[macro_use] extern crate rocket;

mod api;
mod storage;
mod topic;

#[cfg(test)]
mod tests;

#[launch]
fn rocket() -> _ {

    // DI management
    let topic_service = Arc::new(TopicService::new());
    let publisher_service = PublisherService::new(topic_service.clone());

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
        //.manage(di_manager)
        .manage(topic_service)

        .mount("/", FileServer::from(relative!("src/broker/web")))
}