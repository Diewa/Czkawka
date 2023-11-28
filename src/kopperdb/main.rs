#[macro_use] extern crate rocket;

mod api;
mod kopper;
mod brass;
mod stats;

#[cfg(test)]
mod tests;

const KOPPERDB_FOLDER: &str = "kopper_database";
const BRASSDB_FOLDER: &str = "brass_database";
const SEGMENT_SIZE: usize = 4096; 

use api::{read_kopper, read_brass,
          write_kopper, write_brass,
          get_stats, 
          create_kopper, 
          create_brass,
          create_stats};

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![read_kopper, read_brass, write_kopper, write_brass, get_stats])
        .manage(create_stats())
        .manage(create_brass(BRASSDB_FOLDER, SEGMENT_SIZE).expect("Can't create Brass"))
        .manage(create_kopper(KOPPERDB_FOLDER, SEGMENT_SIZE).expect("Can't create Kopper")) // Shared state accessible by ref in all endpoints. Must be Send + Sync
}


