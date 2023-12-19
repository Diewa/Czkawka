
#[macro_use] extern crate rocket;

// Mods
mod api;
mod storage;
mod topic;
mod router;

#[cfg(test)]
mod tests;

use router::router;

#[launch]
fn rocket() -> _ {
    router()
}