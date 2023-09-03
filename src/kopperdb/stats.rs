use plotters::prelude::*;
use std::{error::Error, sync::Mutex};


pub struct Stats {
    pub read_counter: Mutex<Vec<usize>>,
    pub write_counter: Mutex<Vec<usize>>,
    pub size: Mutex<Vec<usize>>
}

impl Stats {
    pub fn new() -> Self {
        Stats {
            read_counter: Mutex::new(vec![]),
            write_counter: Mutex::new(vec![]),
            size: Mutex::new(vec![])
        }
    }
}

const OUT_FILE_NAME: &'static str = "stats.png";
const RESOLUTION_QUALITY: usize = 4;

pub fn draw(data: &Vec<usize>, label: &str, unit: &str) -> Result<(), Box<dyn Error>> {

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

    // Create chart
    let mut chart = ChartBuilder::on(&root)
        .set_label_area_size(LabelAreaPosition::Left, 40)
        .set_label_area_size(LabelAreaPosition::Bottom, 40)
        .set_label_area_size(LabelAreaPosition::Right, 40)
        .caption(label, ("sans-serif", 20))
        .build_cartesian_2d(
            (0usize..data.len()).into_segmented(), 
            0usize..(max + 10).min(100000))?;
    
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

#[cfg(test)]
mod test {

    fn get_data(length: usize) -> Vec<usize> {
        let mut ret = vec![];
        for i in 0..length {
            ret.push((i as f64 * (i as f64).sin()).abs() as usize);
        }
        ret
    }

    #[test]
    fn run_stats() {
        super::draw(&get_data(1000), "test", "us").expect("Can't run draw()");
    }

}