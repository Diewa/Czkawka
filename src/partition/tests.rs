 
use rand::{distributions::Alphanumeric, Rng};

use super::partition::{Partition, PartitionError};

const DB_PATH: &str = "testfiles/partition";

fn random_str_with_size(size: usize) -> String {
    Rng::sample_iter(rand::thread_rng(), &Alphanumeric)
    .take(size)
    .map(char::from)
    .collect()
}

fn new_path() -> String {
    format!("{}/{}", DB_PATH, random_str_with_size(20))
}

#[test]
fn produce_and_consume_one() -> Result<(), PartitionError> {
    let mut p = Partition::new(&new_path())?;

    let offset = p.produce("MyNewCrazyValue")?;
    let entries = p.consume(offset)?;
    
    assert_eq!(entries.next()?.unwrap().value, "MyNewCrazyValue");
    assert!(entries.next()?.is_none());
    Ok(())
}

#[test]
fn consume_empty() -> Result<(), PartitionError> {
    let p = Partition::new(&new_path())?;
    let entries = p.consume(0);
    assert_eq!(entries.unwrap_err().to_string(), PartitionError::BadOffset(0).to_string());
    Ok(())
}

#[test]
fn consume_wrong_offset() -> Result<(), PartitionError> {
    let mut p = Partition::new(&new_path())?;
    p.produce("MyNewCrazyValue")?;

    let err = p.consume(100).unwrap_err();
    assert_eq!(err.to_string(), PartitionError::BadOffset(100).to_string());
    Ok(())
}

#[test]
fn produce_many_consume_loop() -> Result<(), PartitionError> {
    let mut p = Partition::new(&new_path())?;

    p.produce("1")?;
    p.produce("2")?;
    p.produce("3")?;
    let entries = p.consume(0)?;
    
    let mut sum = 0;
    while let Some(entry) = entries.next()? {
        sum += entry.value.parse::<i32>().unwrap();
    }

    assert_eq!(sum, 6);
    Ok(())
}

#[test]
fn first_offset_happy() -> Result<(), PartitionError> {
    let mut p = Partition::new(&new_path())?;
    p.produce("3")?;

    assert_eq!(p.first_offset()?, 0);
    Ok(())
}

#[test]
fn first_offset_empty() -> Result<(), PartitionError> {
    let p = Partition::new(&new_path())?;
    let err = p.first_offset().unwrap_err();

    assert_eq!(err.to_string(), PartitionError::NoFirstOffset.to_string());
    Ok(())
}

#[test]
fn simple_recover() -> Result<(), PartitionError> {
    let path = new_path();
    let mut p = Partition::new(&path)?;

    let offset = p.produce("asd")?;

    let p: Partition = Partition::new(&path)?;

    assert_eq!(p.consume(offset)?.next()?.unwrap().value, "asd");
    Ok(())
}

#[test]
fn recover_empty() -> Result<(), PartitionError> {
    let path = new_path();
    Partition::new(&path)?;
    let mut p = Partition::new(&path)?;
    let o = p.produce("ASD")?;
    assert_eq!(p.consume(o)?.next()?.unwrap().value, "ASD");
    Ok(())
}

#[test]
fn requested_offset_is_first_after_consuming() -> Result<(), PartitionError> {
    let mut p = Partition::new(&new_path())?;

    p.produce("A")?;
    p.produce("B")?;
    let offset = p.produce("C")?;

    assert_eq!(p.consume(offset)?.next()?.unwrap().value, "C");
    Ok(())
}

#[test]
fn multiple_segments() -> Result<(), PartitionError> {
    let mut p = Partition::new(&new_path())?;

    // Currently SEG_SIZE is 4096. Add few entries to trigger second segment creation
    p.produce(std::str::from_utf8(&vec![b'A'; 1200]).unwrap())?;
    p.produce(std::str::from_utf8(&vec![b'A'; 1200]).unwrap())?;
    p.produce(std::str::from_utf8(&vec![b'A'; 1200]).unwrap())?;
    let offset: u64 = p.produce(std::str::from_utf8(&vec![b'A'; 1200]).unwrap())?; // This one should be in next seg

    // Consume the last one
    let collection = p.consume(offset)?;
    collection.next()?;

    // Since last entry should be in the new segment, the amount of bytes required to load from disk 
    // and parse should be small, around the size of last entry's value. 
    assert!(collection.size_read() < 1300);
    Ok(())
}

#[test]
fn recover_multiple_segments() -> Result<(), PartitionError> {
    let path = new_path();
    let mut p = Partition::new(&path)?;

    // Currently SEG_SIZE is 4096. Add few entries to trigger second segment creation
    p.produce(std::str::from_utf8(&vec![b'A'; 1200]).unwrap())?;
    p.produce(std::str::from_utf8(&vec![b'A'; 1200]).unwrap())?;
    p.produce(std::str::from_utf8(&vec![b'A'; 1200]).unwrap())?;
    let offset: u64 = p.produce(std::str::from_utf8(&vec![b'B'; 1200]).unwrap())?; // This one should be in next seg

    let p = Partition::new(&path)?;
    assert_eq!(p.consume(offset)?.next()?.unwrap().value.chars().nth(0), Some('B'));
    Ok(())
}

