use rocket::fs::{FileServer, relative};

#[macro_use] extern crate rocket;

mod api;
mod storage;

#[launch]
fn rocket() -> _ {
    rocket::build()
        .configure(rocket::Config::figment().merge(("port", 8081)))
        .mount("/publish", routes![
            api::receiver::publish_message,
            api::receiver::get_offset
        ])

        .mount("/admin", routes![
            api::admin::create_topic
        ])
        
        .mount("/", FileServer::from(relative!("web")))
}