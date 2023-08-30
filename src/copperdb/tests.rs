use std::{sync::Arc, collections::HashMap};

use super::api::*;

use rocket::Config;

fn client() -> rocket::local::blocking::Client {
    let config = Config {
        log_level: rocket::config::LogLevel::Off,
        ..Config::debug_default()
    };
    
    let client = rocket::local::blocking::Client::untracked(
        rocket::custom(&config)
            .mount("/", routes![read, write])
            .manage(create_shared_state().expect("Can't create copper")) 
    )
    .expect("Could not build the client");
    client
}


#[test]
fn test_simple_write() {
    // Setup
    let client = client();

    // Act
    let response = client.get("/write/lowkey/highvalue").dispatch();
 
    // Test
    assert_eq!(response.status(), rocket::http::Status::Ok);
    assert_eq!(&response.into_string().unwrap(), "{\"error\":\"OK\"}");
}

#[test]
fn test_write_read() {
    let client = client();

    client.get("/write/lowkey/highvalue").dispatch();
    let read_response = client.get("/read/lowkey").dispatch();

    assert_eq!(read_response.status(), rocket::http::Status::Ok);
    assert_eq!(&read_response.into_string().unwrap(), "{\"value\":\"highvalue\",\"error\":\"OK\"}");
}

#[test]
fn mutex_test() {

    let mut map = HashMap::new();

    const N: i32 = 10000;
    for i in 0..N {
        map.insert(i.to_string(), i);
    }

    let arc = Arc::new(std::sync::Mutex::new(map));

    let start = std::time::Instant::now();
    
    let mut threads = vec![];
    
    for _ in 0..10 {
        let arc_tmp = arc.clone();
        threads.push(std::thread::spawn(move || {
            for i in 0..N {
                let lock = arc_tmp.lock().unwrap();
                lock.get(&i.to_string());
            }
        }));
    }

    for _ in 0..10 {
        let arc_tmp = arc.clone();
        threads.push(std::thread::spawn(move || {
            for i in 0..N {
                let mut lock = arc_tmp.lock().unwrap();
                lock.insert(i.to_string(), i);
            }
        }));
    }
    
    for thread in threads {
        thread.join().unwrap();
    }

    println!("Len: {}", arc.lock().unwrap().len());
    println!("Time: {}", start.elapsed().as_millis());

}