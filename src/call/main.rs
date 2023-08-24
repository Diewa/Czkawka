use std::time::Instant;

use reqwest::{self, Client};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    const REQUEST_COUNT: usize = 1000;
    let mut response_checklist = vec![false; REQUEST_COUNT];

    let timer = Instant::now();
    let client = Client::new();
    
    for i in 0..REQUEST_COUNT {

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

    println!("Received: {} / {}", sum, REQUEST_COUNT);
    println!("Time elapsed is: {:?}", duration);

    Ok(())
}