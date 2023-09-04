use std::{sync::{Arc, Mutex}, collections::HashMap, time::Instant};

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
            .manage(create_kopper().expect("Can't create opper")) 
            .manage(create_stats())
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

fn channel_mpsc_perf(thread_count: usize, mgs_count: usize) {

    use std::sync::mpsc;

    let (tx, rx) = mpsc::channel();

    let time = Instant::now();
    let mut threads = vec![];
    for _ in 0..thread_count {
        let tx_tmp = tx.clone();
        threads.push(std::thread::spawn(move || {
            for msg_index in 0..mgs_count {
                tx_tmp.send(msg_index.to_string()).unwrap();
            }
        }));
    }
  
    let recv_thread = std::thread::spawn(move || {
        let mut received_data = vec![];
        for _ in 0..thread_count * mgs_count {
            received_data.push(rx.recv().unwrap());
        }

        assert_eq!(thread_count * mgs_count, received_data.len());
    });

    for thread in threads {
        thread.join().unwrap();
    }

    println!("Sending time: {}", time.elapsed().as_millis());

    recv_thread.join().unwrap();
    println!("Total (with receive) time: {}", time.elapsed().as_millis());

}

fn mutex_mpsc_perf(thread_count: usize, mgs_count: usize) {
    
    let mutex = Arc::new(Mutex::new(vec![]));
    
    let time = Instant::now();
    let mut threads = vec![];
    for _ in 0..thread_count {
        let m = mutex.clone();
        threads.push(std::thread::spawn(move || {
            for msg_index in 0..mgs_count {
                m.lock().unwrap().push(msg_index.to_string());
            }
        }));
    }

    for thread in threads {
        thread.join().unwrap();
    }

    assert_eq!(thread_count * mgs_count, mutex.lock().unwrap().len());
    println!("Total time: {}", time.elapsed().as_millis());
}

#[test]
fn channels_vs_mutex() {
    const THREADS: usize = 10;
    const MESSAGES: usize = 100000;

    println!("CHANNELS:");
    channel_mpsc_perf(THREADS, MESSAGES);
    println!("MUTEX:");
    mutex_mpsc_perf(THREADS, MESSAGES);
}

#[test]
fn sync_of_arc() {
    let a = Arc::new(1);

    let b = a.clone();
    std::thread::spawn(move || {
        println!("{}", b);
    });

    std::thread::spawn(|| {
        
    });
}