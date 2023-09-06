use std::time::Instant;

use rocket::State;
use rocket::serde::json::Json;
use rocket::fs::NamedFile;
use serde::Serialize;

use crate::kopper::Kopper;
use crate::stats::{Stats, self, Stat};

#[derive(Serialize)]
pub struct ReadResponse {
    value: String,
    error: String
}

#[derive(Serialize)]
pub struct WriteResponse {
    error: String
}

#[get("/read/<key>")]
pub fn read(key: String, db: &State<Kopper>, stats: &State<Stats>) -> Json<ReadResponse> {
    let timer = Instant::now();
    
    let mut response = ReadResponse { 
        value: String::from(""), 
        error: String::from("OK") 
    };
    
    match db.read(key) {
        Ok(value_option) => {
            match value_option {
                Some(value) => {
                    response.value = value
                },
                None => {
                    response.error = "No such thing in database".to_string()
                }
            }
        },
        Err(err) => {
            response.error = err.to_string()
        }
    };
    
    stats.send(Stat::ReadTime(timer.elapsed().as_nanos()));
    Json(response)
}

#[get("/write/<key>/<value>")]
pub fn write(key: String, value: String, db: &State<Kopper>, stats: &State<Stats>) -> Json<WriteResponse> {
    let timer = Instant::now();

    let result = match db.write(key, value) {
        Ok(size) => {
            stats.send(Stat::Size(size as u128));
            "OK".to_string()
        },
        Err(err) => format!("Error writing to database! ({})", err)
    };

    stats.send(Stat::WriteTime(timer.elapsed().as_nanos()));
    Json(WriteResponse { error: result.to_string() })
}

#[get("/stats/<read_or_write>")]
pub async fn get_stats(read_or_write: String, stats: &State<Stats>) -> Option<NamedFile> {
    
    if read_or_write.eq("read") {
        let read_counter = stats.counters.read_counter.lock().unwrap();
        stats::draw(&*read_counter, "Reads", "us").expect("Drawing");
    }
    else if read_or_write.eq("write") {
        let write_counter = stats.counters.write_counter.lock().unwrap();
        stats::draw(&*write_counter, "Writes", "us").expect("Drawing");
    }
    else if read_or_write.eq("size") {
        let size_metric = stats.counters.size.lock().unwrap();
        stats::draw(&*size_metric, "Size", "KB").expect("Drawing");
    }
    else {
        return None;
    }

    return NamedFile::open(std::path::Path::new("stats.png")).await.ok()
}

pub fn create_kopper(path: &str) -> Result<Kopper, std::io::Error> {
    Kopper::start(path)
}

pub fn create_stats() -> Stats {
    let (stats, mut aggregator) = Stats::create();

    // This thread exits when last stat Sender is dropped - so when Stats are dropped
    std::thread::spawn(move || {
        aggregator.run();
    });

    stats
}