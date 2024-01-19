
#[macro_use] extern crate rocket;

// Mods
mod api;
mod storage;
mod topic;
mod router;

#[cfg(test)]
mod tests;

use router::router;

const KOPPERDB_FOLDER: &str = "kopper_database";
const PORT: u16 = 8081;

#[launch]
fn rocket() -> _ {
    router(&rocket::config::Config { port: PORT, ..Default::default()}, KOPPERDB_FOLDER)
}