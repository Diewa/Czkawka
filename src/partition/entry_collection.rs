use std::{rc::Rc, cell::RefCell, io::Cursor};

use super::partition::PartitionError;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct PartitionEntry {
    offset: u64,
    timestamp: u64,
    value: String
}

#[derive(Clone)]
pub struct EntryCollection {
    cursor: Rc<RefCell<Cursor<Vec<u8>>>>
}

impl std::io::Read for EntryCollection {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.cursor.borrow_mut().read(buf)
    }
}

impl EntryCollection {

    pub fn new(data: Vec<u8>) -> Self {
        EntryCollection { 
            cursor: Rc::new(RefCell::new(
                Cursor::new(data)
            )) 
        }
    }

    pub fn next(&self) -> Result<PartitionEntry, PartitionError> {
        Ok(bincode::deserialize_from(self.clone())?)
    }
}

#[test]
fn test_serde() {
    let p0 = PartitionEntry { offset: 0, timestamp: 4, value: format!("p0") };
    let p1 = PartitionEntry { offset: 1, timestamp: 5, value: format!("p1") };
    let p2 = PartitionEntry { offset: 2, timestamp: 6, value: format!("p2") };

    let mut vec: Vec<u8> = vec![]; 
    vec.append(&mut bincode::serialize(&p0).unwrap());
    vec.append(&mut bincode::serialize(&p1).unwrap());
    vec.append(&mut bincode::serialize(&p2).unwrap());
    
    let collection = EntryCollection::new(vec);

    let new_p0 = collection.next().unwrap();
    assert_eq!(p0.offset, new_p0.offset);
    assert_eq!(p0.timestamp, new_p0.timestamp);
    assert_eq!(p0.value, new_p0.value);
    let new_p1 = collection.next().unwrap();
    assert_eq!(p1.offset, new_p1.offset);
    assert_eq!(p1.timestamp, new_p1.timestamp);
    assert_eq!(p1.value, new_p1.value);
    let new_p2 = collection.next().unwrap();
    assert_eq!(p2.offset, new_p2.offset);
    assert_eq!(p2.timestamp, new_p2.timestamp);
    assert_eq!(p2.value, new_p2.value);
}

#[test]
fn ser_str_de_string() {
    
    #[derive(Debug, serde::Serialize)]
    struct A<'a> { s: &'a str }

    #[derive(Debug, serde::Deserialize)]
    struct B     { _s: String }

    let a = A { s: "ASD" };
    
    let vec = bincode::serialize(&a).unwrap();
    println!("{:?}", bincode::deserialize::<B>(&vec));
}
