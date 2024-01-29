use std::{collections::BTreeMap, fs::File, io::{self, Write, Seek, Read}, time::SystemTime};

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
from_error!(PartitionError::Internal, std::io::Error, std::time::SystemTimeError);

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
        match Partition::recover(path)? {
            Some(partition) => {
                
                // 1. The log is already on disk 
                Ok(partition)
            }
            None => {

                // 2. The log is not on the disk, or there's only an empty file
                Partition::create_new(path)
            }
        }
    }

    ///
    /// Adds a new message to the end of partition
    ///
    pub fn produce(&mut self, value: &str) -> Result<Offset, PartitionError> {
        
        let timestamp = 
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)?
                .as_secs();

        let offset = self.next_offset;
        let new_partition_entry = PartitionEntry::serialize(offset, timestamp, value)?;

        // Create metadata for the entry. We can unwrap here because at least one entry always exists
        let mut last_index_entry = self.index.last_entry().unwrap();

        // Create new segment if needed
        if last_index_entry.get_mut().size + new_partition_entry.len() > SEG_LENGTH {
            let file = last_index_entry.get_mut().file.try_clone()?;
            self.index.insert(offset, IndexEntry {
                file,
                address: 0,
                size: 0
            });

            last_index_entry = self.index.last_entry().unwrap();
        }

        // Get to end of file, and write our bytes there
        last_index_entry.get_mut().file.seek(io::SeekFrom::End(0))?;
        last_index_entry.get_mut().file.write_all(&new_partition_entry)?;
        last_index_entry.get_mut().size += new_partition_entry.len();
        
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
        
        Ok(EntryCollection::new(buffer, offset))
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

        let first_file_path = format!("{}/0", path);

        // Create first file (or open if it already exists)
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

    fn recover(path: &str) -> Result<Option<Self>, PartitionError> {

        // Create a folder if it doesn't exist
        std::fs::create_dir_all(path)?;

        let mut index = BTreeMap::new();
        let mut highest_offset = 0;

        // Check all files in the folder
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

            let collection = EntryCollection::new(buf, 0);

            let mut last_seg_cutoff = 0;
            let mut current_seg_size = 0;
            let mut first_offset_of_segment = None;
            while let Some(entry) = collection.next()? {

                if first_offset_of_segment.is_none() {
                    first_offset_of_segment = Some(entry.offset);
                }

                if entry.offset > highest_offset {
                    highest_offset = entry.offset;
                }

                if collection.size_read() - last_seg_cutoff <= SEG_LENGTH {
                    current_seg_size = collection.size_read() - last_seg_cutoff;
                    continue;
                }
                
                // Cut of a new segment, i.e. create a new index entry
                index.insert(first_offset_of_segment.unwrap(), IndexEntry {
                    file: file.try_clone()?,
                    address: last_seg_cutoff,
                    size: current_seg_size,
                });

                last_seg_cutoff = current_seg_size;
                current_seg_size = collection.size_read() - last_seg_cutoff;
                first_offset_of_segment = Some(entry.offset);
            }

            if let Some(offset) = first_offset_of_segment {
                index.insert(offset, IndexEntry {
                    file: file.try_clone()?,
                    address: last_seg_cutoff,
                    size: current_seg_size,
                });
            }
        }


        // Failing to recover partition can happen when:
        // 1. There are no partition files
        // 2. A file exist (Partition's been created before) but nothing's been produced yet
        if index.is_empty() {
            return Ok(None);
        }

        Ok(Some(Partition {
            index,
            next_offset: highest_offset + 1,
        }))
    }
}