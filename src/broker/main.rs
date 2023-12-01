use rocket::fs::{FileServer, relative};
use crate::topic::topic_service::TopicService;

#[macro_use] extern crate rocket;

mod api;
mod storage;
mod topic;

#[launch]
fn rocket() -> _ {
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
        .manage(TopicService::new())
        
        .mount("/", FileServer::from(relative!("src/broker/web")))
}