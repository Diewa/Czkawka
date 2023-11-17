use plotters::prelude::*;
use std::{error::Error, sync::{self, Mutex, mpsc::channel, Arc}};

pub struct StatsAggregator {
    receiver: sync::mpsc::Receiver<Stat>,
    counters: Arc<Counters>
}

pub struct Counters {
    pub read_counter: Mutex<Vec<u128>>,
    pub write_counter: Mutex<Vec<u128>>,
    pub size: Mutex<Vec<u128>>,
}

impl Counters {
    pub fn new() -> Self {
        Counters {
            read_counter: Mutex::default(),
            write_counter: Mutex::default(),
            size: Mutex::default()
        }
    }
}

impl StatsAggregator {
    pub fn run(&mut self) {
        loop {
            match self.receiver.recv() {
                Ok(stat) => match stat {
                    Stat::ReadTime(time) => self.counters.read_counter.lock().unwrap().push(time),
                    Stat::WriteTime(time) => self.counters.write_counter.lock().unwrap().push(time),
                    Stat::Size(size) => self.counters.size.lock().unwrap().push(size),
                }

                // Sender disconnected - stop the thread
                Err(_) => break
            }
        }
    }
}

pub enum Stat {
    ReadTime(u128),
    WriteTime(u128),
    Size(u128)
}

pub struct Stats {
    sender: sync::mpsc::Sender<Stat>,
    pub counters: Arc<Counters>,
}

impl Stats {
    pub fn create() -> (Stats, StatsAggregator) {
        let (tx, rx) = channel();
        let counters = Arc::new(Counters::new());
        (Stats {
            sender: tx,
            counters: counters.clone()
        },
        StatsAggregator {
            receiver: rx,
            counters
        })
    }

    pub fn send(&self, stat: Stat) {
        self.sender.send(stat).unwrap();
    }
}

const OUT_FILE_NAME: &'static str = "stats.png";
const RESOLUTION_QUALITY: usize = 4;

pub fn draw(data: &Vec<u128>, label: &str, unit: &str) -> Result<(), Box<dyn Error>> {

    // Find the biggest datapoint to use as height of graph
    let max = match data.iter().max() {
        Some(max) => *max,
        None => 1,
    };
    
    // Setup bitmap
    let root = BitMapBackend::new(
        OUT_FILE_NAME, 
        (640.max(RESOLUTION_QUALITY * data.len()) as u32,
         640 as u32)).into_drawing_area();

    // Background
    root.fill(&WHITE)?;

    // Calculate p50, p95, p99
    let mut sorted = data.clone();
    sorted.sort();
    let p50: u128 = sorted[sorted.len() * 50 / 100] / 1000;
    let p95 = sorted[sorted.len() * 95 / 100] / 1000;
    let p99 = sorted[sorted.len() * 99 / 100] / 1000;

    // Create chart
    let mut chart = ChartBuilder::on(&root)
        .set_label_area_size(LabelAreaPosition::Left, 40)
        .set_label_area_size(LabelAreaPosition::Bottom, 40)
        .set_label_area_size(LabelAreaPosition::Right, 40)
        .caption(format!("{label}, p50: {p50}{unit}, p95: {p95}{unit}, p99: {p99}{unit}"), ("sans-serif", 20))
        .build_cartesian_2d(
            (0usize..data.len()).into_segmented(), 
            0u128..(max + 10))?;
    
    chart
        .configure_mesh()
        .y_label_formatter(&|x| format!("{}{}", x / 1000, unit))
        .draw()?;

    chart.draw_series(
        (0usize..) // range [0 to data.len() - 1]
            .zip(data.iter())
            .map(|(x, y)| {

        let x0 = SegmentValue::Exact(x);
        let x1 = SegmentValue::Exact(x + 1);

        // Points: [bottom_left: (x, y), top_right: (x, y)]
        let mut bar = Rectangle::new(
            [(x0, 0), (x1, *y)], 
            RED.mix(0.5).filled()
        );

        bar.set_margin(0, 0, 1, 1);
        bar
    }))?;

    // To avoid the IO failure being ignored silently, we manually call the present function
    root.present().expect("Unable to write result to file");
    Ok(())
}