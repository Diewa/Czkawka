#[macro_use] extern crate rocket;

mod api;

#[launch]
fn rocket() -> _ {
    rocket::build()
        .configure(rocket::Config::figment().merge(("port", 8081)))
        .mount("/publish", routes![
            api::receiver::publish_message,
            api::receiver::get_offset
        ])
}