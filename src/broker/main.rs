#[macro_use] extern crate rocket;

mod receiver;
mod storage;

#[launch]
fn rocket() -> _ {
    rocket::build()
        .configure(rocket::Config::figment().merge(("port", 8081)))
        .mount("/publish", routes![
            receiver::receiver_endpoint::publish_message,
        ])
}