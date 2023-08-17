#[macro_use] extern crate rocket;

mod call_controller;

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![call_controller::index])
}