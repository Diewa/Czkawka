use std::{cell::RefCell, collections::BTreeMap, fs::File, io::{self, Write, Seek, Read}, rc::Rc, time::SystemTime};

use bincode::error::{EncodeError, DecodeError};
use kopperdb::from_error;

use crate::partition::entry_collection::*;

pub type Offset = u64;

const SEG_LENGTH: usize = 4096;

#[derive(thiserror::Error, Debug)]
pub enum PartitionError {
    #[error("There are no messages in the partition")]
    NoFirstOffset,

    #[error("Offset {0} does not exist")]
    BadOffset(Offset),

    #[error(transparent)]
    Internal(#[from] anyhow::Error)
}

// Awesome macro to be able to turn errors into PartitionError::Internal using "?"
from_error!(PartitionError::Internal, std::io::Error, std::time::SystemTimeError, EncodeError, DecodeError);

///
/// Partition represents the in-memory index of an on-disk log
///
pub struct Partition {
    index: BTreeMap<Offset, IndexEntry>,
    next_offset: Offset
}

struct IndexEntry {
    file: File,
    address: usize,
    size: usize,
}

impl Partition {
    pub fn new(path: &str) -> Result<Self, PartitionError> {
        // There are two possible states when creating Partition:

        // 1. The log is already on disk        
        if Partition::partition_exists_at_path(path)? {
            return Partition::recover(path);
        }
        
        // 2. We need to create a fresh log
        Partition::create_new(path)
    }

    ///
    /// Adds a new message to the end of partition
    ///
    pub fn produce(&mut self, value: &str) -> Result<Offset, PartitionError> {
        
        // Create metadata for the entry. We can unwrap here because at least one entry always exists
        let mut entry = self.index.last_entry().unwrap();

        let timestamp = 
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)?
                .as_secs();

        let offset = self.next_offset;

        let serialized_partition_entry = PartitionEntry::serialize(offset, timestamp, value)?;

        // Get to end of file, and write our bytes there
        entry.get_mut().file.seek(io::SeekFrom::End(0))?;
        entry.get_mut().file.write_all(&serialized_partition_entry)?;

        entry.get_mut().size += serialized_partition_entry.len();
        self.next_offset = offset + 1;
        Ok(offset)
    }

    ///
    /// Retrieves a number messages at the specified offset. The amount of messages
    /// is the difference between given offset and next in the history tree
    ///
    pub fn consume(&self, offset: Offset) -> Result<EntryCollection, PartitionError> {

        if offset >= self.next_offset {
            return Err(PartitionError::BadOffset(offset));
        }

        // Find the address of offset *equal or smaller* than requested
        let index_entry = 
            self.index
                .range(..=offset)
                .next_back()
                .ok_or_else(|| PartitionError::BadOffset(offset))?.1;

        // Prepare info about segment of the file we're trying to read
        let mut file = index_entry.file.try_clone()?;
        let mut buffer = vec![0u8; index_entry.size];

        // Read the segment into memory
        file.seek(io::SeekFrom::Start(index_entry.address as u64))?;
        file.read_exact(&mut buffer)?;
        
        Ok(EntryCollection::new(buffer))
    }

    ///
    /// Returns the offset of the earliest available message
    ///
    pub fn first_offset(&self) -> Result<Offset, PartitionError> {
        if self.next_offset == 0 {
            return Err(PartitionError::NoFirstOffset);
        }

        // It's ok to unwrap because there's always an item in index
        Ok(self.index.first_key_value().unwrap().0.clone())
    }

    // TODO: pub fn set_retention_period(time) {}

    fn partition_exists_at_path(path: &str) -> Result<bool, PartitionError> {
        
        // Does the folder exist?
        if std::path::Path::new(path).exists() {
            return Ok(true);
        }

        std::fs::create_dir_all(path)?;
        Ok(false)
    }

    fn create_new(path: &str) -> Result<Self, PartitionError> {
        // Let's check that
        let first_file_path = format!("{}/0", path);

        // Create a log from scratch
        // Setup the first file
        let first_file = 
            std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open(&first_file_path)?;

        // Create the first index
        let index = IndexEntry {
            file: first_file.try_clone()?,
            address: 0,
            size: 0
        };

        // Put it into the tree
        let mut btree = BTreeMap::new();
        btree.insert(0, index);

        Ok(Partition {
            index: btree,
            next_offset: 0
        })
    }

    fn recover(path: &str) -> Result<Self, PartitionError> {

        let mut index = BTreeMap::new();
        let mut highest_offset = 0;

        for dir_entry in std::fs::read_dir(path)? {

            let path = dir_entry?.path();

            let mut file = 
                std::fs::OpenOptions::new()
                    .read(true)
                    .append(true)
                    .create(true)
                    .open(path)?;

            let mut buf = vec![];
            file.read_to_end(&mut buf)?;

            let collection = EntryCollection::new(buf);

            let mut last_seg_cutoff = 0;
            while let Some(entry) = collection.next()? {

                let length_since_last_cutoff = collection.size_read() - last_seg_cutoff;
                let is_first_entry_of_file = last_seg_cutoff == 0;
                let is_longer_than_seg_length = length_since_last_cutoff > SEG_LENGTH;

                if !(is_first_entry_of_file || is_longer_than_seg_length) {
                    continue;
                }
                
                // Cut of a new segment, i.e. create a new index entry
                index.insert(entry.offset, IndexEntry {
                    file: file.try_clone()?,
                    address: last_seg_cutoff,
                    size: length_since_last_cutoff,
                });

                last_seg_cutoff = collection.size_read();

                if entry.offset > highest_offset {
                    highest_offset = entry.offset;
                }
            }
        }

        Ok(Partition {
            index,
            next_offset: highest_offset + 1,
        })
    }
}