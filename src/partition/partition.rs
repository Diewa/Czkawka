use std::{collections::BTreeMap, fs::File, io::{self, Write, Seek, Read}, time::SystemTime, rc::Rc, cell::RefCell};

use kopperdb::from_error;

use crate::partition::entry_collection::*;

type Offset = u64;

struct IndexEntry {
    file: Rc<RefCell<File>>,
    address: usize,
}

struct CurrentFile {
    file: Rc<RefCell<File>>,
    offset: Offset
}

///
/// Partition represents the in-memory index of an on-disk log
///
pub struct Partition {
    path: String,
    index: BTreeMap<Offset, IndexEntry>,
    current_file: CurrentFile
}

#[derive(thiserror::Error, Debug)]
pub enum PartitionError {
    
    #[error(transparent)]
    Internal(#[from] anyhow::Error)
}

// Awesome macro to be able to turn errors into PartitionError::Internal using "?"
from_error!(PartitionError::Internal, std::io::Error, std::time::SystemTimeError, bincode::Error);

// Derive from serde to be able to use bincode and easily turn struct into bytes and vice-versa
#[derive(Debug, serde::Serialize)]
struct DiskEntry<'a> {
    offset: u64,
    timestamp: u64,
    value: &'a str
}

impl Partition {
    pub fn new(path: &str) -> Result<Self, PartitionError> {
        // There are two possible states when creating Partition:
        // 1. The log is already on disk
        // 2. We need to create a fresh log
        
        // Let's check that
        if Partition::partition_exists_at_path(path)? {
            return Partition::recover(path);
        }
        
        // Create a log from scratch
        // Setup the first file
        let first_file = Rc::new(RefCell::new(
            std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open("0")?));

        // Create the first index
        let index = IndexEntry {
            file: first_file.clone(),
            address: 0,
        };

        // Put it into the tree
        let mut btree = BTreeMap::new();
        btree.insert(0, index);

        Ok(Partition {
            path: path.to_owned(),
            index: btree,
            current_file: CurrentFile { file: first_file, offset: 0}
        })
    }

    ///
    /// Adds a new message to the end of partition
    ///
    pub fn produce(&mut self, value: &str) -> Result<Offset, PartitionError> {
        
        // Create metadata for the entry
        let offset = self.current_file.offset;

        let timestamp = 
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)?
                .as_secs();

        let header = DiskEntry {
            offset,
            timestamp,
            value
        };

        let bytes = bincode::serialize(&header)?;

        // Get to end of file, and write our bytes there
        self.current_file.file.borrow_mut().seek(io::SeekFrom::End(0))?;
        self.current_file.file.borrow_mut().write_all(&bytes)?;
        self.current_file.offset += 1;
        Ok(offset)
    }

    ///
    /// Retrieves a number messages at the specified offset. The amount of messages
    /// is the difference between given offset and next in the history tree
    ///
    pub fn consume(&self, offset: Offset) -> Result<EntryCollection, PartitionError> {
        use std::ops::Bound::{Included, Excluded, Unbounded};

        // Find the address of offset closest but smaller than requested
        let index_entry_before = 
            self.index.range((Unbounded, Included(offset))).next_back().unwrap();

        // Find the address of offset closest but bigger than requested
        let index_entry_after = 
            self.index.range((Excluded(offset), Unbounded)).next().unwrap();

        // Prepare info about segment of the file we're trying to read
        let segment_start_address = index_entry_before.1.address;
        let segment_length = index_entry_after.1.address - index_entry_before.1.address;
        let file = index_entry_before.1.file.clone();

        let mut buffer = vec![0u8; segment_length];

        // Read the segment into memory
        file.borrow_mut().seek(io::SeekFrom::Start(segment_start_address as u64))?;
        file.borrow_mut().read_exact(&mut buffer)?;
        
        Ok(EntryCollection::new(buffer))
    }

    ///
    /// eturns the offset of the earliest available message
    ///
    pub fn first_offset(&self) -> Result<Offset, PartitionError> {
        todo!()
    }

    // TODO: pub fn set_retention_period(time) {}

    fn partition_exists_at_path(path: &str) -> Result<bool, PartitionError> {
        
        // Does the folder exist?
        match std::fs::create_dir_all(path) {
            // Successfully created a folder, so it didn't exist before
            Ok(_) => return Ok(false),

            Err(e) => {
                // If any other error than AlreadyExists occured, return error
                if e.kind() != io::ErrorKind::AlreadyExists {
                    return Err(e.into());
                }
                
                // Folder already exists
                return Ok(true);
            }
        }
    }

    fn recover(path: &str) -> Result<Self, PartitionError> {
        todo!()
    }
}