#[macro_use] extern crate rocket;

mod api;
mod copper;

#[cfg(test)]
mod tests;

use api::{read, write, create_shared_state};

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![read, write])
        .manage(create_shared_state()) // Shared state accessible by ref in all endpoints. Must be Send + Sync
}


