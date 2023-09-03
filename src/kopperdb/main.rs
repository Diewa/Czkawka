#[macro_use] extern crate rocket;

mod api;
mod kopper;
mod stats;

#[cfg(test)]
mod tests;


use api::{read, write, get_stats, create_kopper, create_stats};

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![read, write, get_stats])
        .manage(create_kopper().expect("Can't create Kopper")) // Shared state accessible by ref in all endpoints. Must be Send + Sync
        .manage(create_stats())
}


