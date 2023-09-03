use std::time::Instant;

use reqwest::{self, Client};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    send_until(&|x| {

        // Send 1000 requests
        if x == 1000 { return None; }

        if x % 2 == 0 {
            return Some(format!("http://127.0.0.1:8000/write/{}/{}", 1, 1));
        }
        else {
            return Some(format!("http://127.0.0.1:8000/read/{}", 1));
        }
        
    }).await?;
    
    Ok(())
}

async fn send_until(url: &dyn Fn(usize) -> Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();

    for i in 0.. {
        match url(i) {
            Some(url) => {
                client
                    .get(url)
                    .send()
                    .await?
                    .text()
                    .await?;
            },
            None => break
        }
    } 

    Ok(())
}

#[allow(dead_code)]
async fn send_and_verify_n_requests(n: usize) -> Result<(), Box<dyn std::error::Error>> {
    let mut response_checklist = vec![false; n];

    let timer = Instant::now();
    let client = Client::new();
    
    for i in 0..n {

        let response = client
            .get(format!("http://127.0.0.1:8000/echo/{}/asd", i))
            .send()
            .await?
            .text()
            .await?;
    
        // Response format: "{id} : asd"
        let id = response.split(' ').collect::<Vec<&str>>()[0];
        
        let parsed_id: usize = id.parse().expect("Can't parse number");

        // Set response index in the checklist
        response_checklist[parsed_id] = true;
    } 

    let duration = timer.elapsed();

    let sum = response_checklist
                        .iter() // Take iterator over Vec<bool>
                        .map(|x| { if *x { 1 } else { 0 } }) // Map true to 1 and false to 0
                        .sum::<usize>(); // Add up all 1s

    println!("Received: {} / {}", sum, n);
    println!("Time elapsed is: {:?}", duration);

    Ok(())
}