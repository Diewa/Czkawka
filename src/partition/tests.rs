use std::{fs::{self, File}, io::Write, sync::Mutex};


#[test]
fn sometest() {
    println!("{}", std::mem::size_of::<Mutex<File>>());
    super::partition::Partition::new("ASD");
}