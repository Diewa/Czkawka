use std::sync::Arc;

use rocket::{Rocket, Build};
use rocket::fs::{FileServer, relative};
use kopperdb::kopper::Kopper;

use crate::api;
use crate::topic::{
    publisher_service::PublisherService,
    topic_service::TopicService
};

const SEGMENT_SIZE: usize = 4000; 

pub fn router(config: &rocket::Config, db_folder: &str) -> Rocket<Build> {
    
    let web_path = relative!("src/web");

    // DI management
    let kopper = Kopper::create(db_folder, SEGMENT_SIZE).expect("Can't create Kopper!");
    let topic_service = Arc::new(TopicService::new(kopper.clone()).expect("Can't create topic service"));
    let publisher_service = Arc::new(PublisherService::new(topic_service.clone(), kopper));
    let templater = Arc::new(api::templater::Templater::new(web_path));

    rocket::custom(config)

        // PUBLISH
        .mount("/publish", routes![
            api::receiver::publish_message,
            api::receiver::get_offset
        ])

        // ADMIN API
        .mount("/", routes![api::admin::web_main])
        .mount("/admin", routes![
            // Htmx endpoints
            api::admin::create_topic,
            api::admin::create_subscriber,
            
            api::admin::module_main,
            api::admin::module_topic,

            // Direct browser URL access helpers
            api::admin::web_main, // /admin
            api::admin::web_topic // /admin/topic/<name>
        ])
        .manage(topic_service)
        .manage(publisher_service)
        
        .manage(templater)
        .mount("/static", FileServer::from(web_path.to_owned() + "/static"))
}