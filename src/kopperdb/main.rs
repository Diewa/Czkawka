#[macro_use] extern crate rocket;

mod api;
mod kopper;
mod stats;

#[cfg(test)]
mod tests;

const DB_FOLDER: &str = "kopper_database";
const SEGMENT_SIZE: u64 = 4096; 

use api::{read, 
          write, 
          get_stats, 
          create_kopper, 
          create_stats};

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![read, write, get_stats])
        .manage(create_stats())
        .manage(create_kopper(DB_FOLDER, SEGMENT_SIZE).expect("Can't create Kopper")) // Shared state accessible by ref in all endpoints. Must be Send + Sync
}


