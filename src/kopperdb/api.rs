use std::time::Instant;

use rocket::State;
use rocket::serde::json::Json;
use rocket::fs::NamedFile;
use serde::Serialize;

use crate::kopper::*;
use crate::brass::*;
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

pub trait Database {
    fn read(&self, key: String) -> std::io::Result<Option<String>>;
    fn write(&self, key: String, value: String) -> std::io::Result<usize>;
}

pub fn read(key: String, db: &impl Database, stats: &State<Stats>) -> Json<ReadResponse> {
    let timer = Instant::now();
    
    let response = match db.read(key.clone()) {

        // Database operation successful
        Ok(value_option) => {
            match value_option {

                // Value exists
                Some(value) => {
                    ReadResponse { 
                        value, 
                        error: String::from("OK") 
                    }
                }

                None => {
                    ReadResponse { 
                        value: "".to_string(),
                        error: format!("{key} does not exist!")
                    }
                }
            }
        },

        Err(err) => {
            ReadResponse { 
                value: "".to_string(),
                error: err.to_string()
            }
        }
    };
    
    stats.send(Stat::ReadTime(timer.elapsed().as_nanos()));
    Json(response)
}

pub fn write(key: String, value: String, db: &impl Database, stats: &State<Stats>) -> Json<WriteResponse> {
    let timer = Instant::now();

    let response = match db.write(key, value) {

        // Database opration successful = write successful
        Ok(size) => {
            stats.send(Stat::Size(size as u128));
            WriteResponse { error: "OK".to_string() }  
        },

        Err(err) => {
            WriteResponse { error: format!("Error while writing! : {}", err) }
        }
    };

    stats.send(Stat::WriteTime(timer.elapsed().as_nanos()));
    Json(response)
}

#[get("/read/<key>")]
pub fn read_kopper(key: String, db: &State<Kopper>, stats: &State<Stats>) -> Json<ReadResponse> {
    read(key, db.inner(), stats)
}

#[get("/write/<key>/<value>")]
pub fn write_kopper(key: String, value: String, db: &State<Kopper>, stats: &State<Stats>) -> Json<WriteResponse> {
    write(key, value, db.inner(), stats)
}

#[get("/read/b/<key>")]
pub fn read_brass(key: String, db: &State<Brass>, stats: &State<Stats>) -> Json<ReadResponse> {
    read(key, db.inner(), stats)
}

#[get("/write/b/<key>/<value>")]
pub fn write_brass(key: String, value: String, db: &State<Brass>, stats: &State<Stats>) -> Json<WriteResponse> {
    write(key, value, db.inner(), stats)
}

#[get("/stats/<read_or_write>")]
pub async fn get_stats(read_or_write: String, stats: &State<Stats>) -> Option<NamedFile> {
    
    match read_or_write.as_str() {
        "read" => {
            let read_counter = stats.counters.read_counter.lock().unwrap();
            stats::draw(&*read_counter, "Reads", "us").expect("Drawing");
        },
        "write" => {
            let write_counter = stats.counters.write_counter.lock().unwrap();
            stats::draw(&*write_counter, "Writes", "us").expect("Drawing");
        },
        "size" => {
            let size_metric = stats.counters.size.lock().unwrap();
            stats::draw(&*size_metric, "Size", "KB").expect("Drawing");
        },
        _ => return None
    }

    return NamedFile::open(std::path::Path::new("stats.png")).await.ok()
}


// TODO: Move the Database trait to another file and implement it in kopper/brass respectively
impl Database for Kopper {
    fn read(&self, key: String) -> std::io::Result<Option<String>> {
        self.read(key)
    }

    fn write(&self, key: String, value: String) -> std::io::Result<usize> {
        self.write(key, value)
    }
}

impl Database for Brass {
    fn read(&self, key: String) -> std::io::Result<Option<String>> {
        self.read(key)
    }

    fn write(&self, key: String, value: String) -> std::io::Result<usize> {
        self.write(key, value)
    }
}

/// Creates a [`Kopper`] instance that can be mounted as a state by Rocket 
pub fn create_kopper(path: &str, segment_size: usize) -> Result<Kopper, KopperError> {
    Kopper::create(path, segment_size)
}

/// Creates a [`Brass`] instance that can be mounted as a state by Rocket 
pub fn create_brass(path: &str, segment_size: usize) -> Result<Brass, BrassError> {
    Brass::create(path, segment_size)
}

/// Creates a [`Stats`] instance that can be mounted as a state by Rocket,
/// as well as starting a [`StatsAggregator`] on a separate thread.
/// 
/// The aggregator thread lifetime is linked to stats. When Stats are destroyed, 
/// so is the aggregator.
pub fn create_stats() -> Stats {
    let (stats, mut aggregator) = Stats::create();

    std::thread::spawn(move || {
        aggregator.run();
    });

    stats
}