use std::cell::Cell;

use kopperdb::from_error;
use bincode::config::Configuration;
use bincode::error::{EncodeError, DecodeError};

use super::partition::{Offset, PartitionError};

// Add bincode errors to PartitionError
from_error!(PartitionError::Internal, EncodeError, DecodeError);

///
/// Represents a single entity that can be fetched from partition.
/// Consists of stored value and metadata
/// 
#[derive(Debug, bincode::BorrowDecode, bincode::Encode)]
pub struct PartitionEntry<'a> {
    pub offset: u64,
    pub timestamp: u64,
    pub value: &'a str
}

impl<'a> PartitionEntry<'a> {

    /// Only to be used by Partition code
    pub(super) fn serialize(offset: u64, timestamp: u64, value: &'a str) -> Result<Vec<u8>, PartitionError> {
        Ok(bincode::encode_to_vec(
            PartitionEntry { offset, timestamp, value }, 
            bincode::config::standard())?
        )
    }
} 

///
/// A set of lazily serialized entries that correspond to one segment fetched from disk.
/// It holds unserialized data fetched from disk and a pointer to the currently read entry.
/// 
/// Entries can be iterated over using `.next()`. The first entry has the offset requested
/// when calling `.consume(offset)`
/// 
#[derive(Debug)]
pub struct EntryCollection {
    data: Vec<u8>,
    address: Cell<usize>,
    first_offset: Offset
}

impl EntryCollection {
    pub fn next(&self) -> Result<Option<PartitionEntry>, PartitionError> {
        if self.address.get() == self.data.len() {
            return Ok(None);
        }

        let (entry, size) = 
            bincode::borrow_decode_from_slice::<PartitionEntry, Configuration>(
                &self.data[self.address.get()..], 
                bincode::config::standard())?;

        self.address.set(self.address.get() + size);

        // Some recursion to get to the first offset
        if entry.offset < self.first_offset {
            return self.next()
        }

        Ok(Some(entry))
    }

    pub(super) fn new(data: Vec<u8>, first_offset: Offset) -> Self {
        EntryCollection {
            data, address: Cell::new(0), first_offset
        }
    }

    pub(super) fn size_read(&self) -> usize {
        self.address.get()
    }
}

#[test]
fn test_entry_collection() {

    // 1. Create few partition entries
    let p1 = PartitionEntry { offset: 1, timestamp: 11, value: "p1" };
    let p2 = PartitionEntry { offset: 2, timestamp: 22, value: "p2" };

    // 2. Serialize then into single vec
    let mut vec: Vec<u8> = vec![]; 
    vec.append(&mut bincode::encode_to_vec(&p1, bincode::config::standard()).unwrap());
    vec.append(&mut bincode::encode_to_vec(&p2, bincode::config::standard()).unwrap());

    // 3. Make it into an EntryCollection
    let ec = EntryCollection::new(vec, 1);
    
    // 4. Profit
    let new_p1 = ec.next().unwrap().unwrap();
    assert_eq!(p1.offset, new_p1.offset);
    assert_eq!(p1.timestamp, new_p1.timestamp);
    assert_eq!(p1.value, new_p1.value);

    let new_p2 = ec.next().unwrap().unwrap();
    assert_eq!(p2.offset, new_p2.offset);
    assert_eq!(p2.timestamp, new_p2.timestamp);
    assert_eq!(p2.value, new_p2.value);

    assert!(ec.next().unwrap().is_none()); // No more elements

}