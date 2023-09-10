use core::num;
use std::{collections::{HashMap, BTreeMap}, sync::Mutex, fs::{File, OpenOptions, self}, io::{Write, Read, self}, os::unix::prelude::FileExt};

pub struct Kopper {
    state: Mutex<SharedState>,
    segment_size: u64,
    path: String
}

type FileIndex = u64;

struct SharedState {
    table: HashMap<String, TableEntry>,
    files: BTreeMap<FileIndex, File>,
    offset: u64,
    current_file_index: FileIndex,
}

struct TableEntry {
    file_index: FileIndex,
    offset: u64,
    len: usize
}

impl Kopper {
    pub fn start(path: &str, segment_size: u64) -> Result<Self, KopperError> {

        // Recover
        let shared_state = SharedState::create(path)?;

        Ok(Kopper { 
            state: Mutex::new(shared_state),
            segment_size,
            path: path.to_owned(),
        })
    }

    #[allow(dead_code)]
    pub fn size(&self) -> u64 {
        self.state.lock().unwrap().offset
    }

    pub fn read(&self, key: String) -> std::io::Result<Option<String>> {
        let state = self.state.lock().unwrap();

        let table_entry = match state.table.get(&key) {
            Some(table_entry) => table_entry,
            None => return Ok(None),
        };

        let file = 
            state.files
                .get(&table_entry.file_index).unwrap() // Can't recover from this. Should panic.
                .try_clone().unwrap(); 

        // TODO: This is OK because files are never deleted. 

        let offset = table_entry.offset;
        let mut buffer = vec![0; table_entry.len];
        drop(state); // Drop mutex early before reading from file

        file.read_exact_at(buffer.as_mut(), offset)?;

        Ok(Some(
            String::from_utf8(buffer)
                .expect("Can't deserialize buffer to string")
        ))
    }

    pub fn write(&self, key: String, value: String) -> std::io::Result<u64> {
        
        let mut state = self.state.lock().unwrap();

        let key_len = key.as_bytes().len();
        let value_len = value.as_bytes().len();

        // 0. Segment file if next entry would exceed max size
        if (key_len + value_len) as u64 + 2 + state.offset > self.segment_size {
            self.cut_off_segment(&mut state);
        }

        // 1. Save in in-memory map
        let entry = TableEntry {
            file_index: state.current_file_index,
            offset: state.offset + key.as_bytes().len() as u64 + 1,
            len: value.as_bytes().len()
        };

        
        state.table.insert(key.clone(), entry);

        // 2. Write to disk
        let mut string_to_save = key;
        string_to_save.push('\0');
        string_to_save.push_str(&value);
        string_to_save.push('\0');
        
        let string_to_save = string_to_save.as_bytes();
        state.files.get(&state.current_file_index).unwrap().write_all(string_to_save)?;

        // Update current offset 
        state.offset += string_to_save.len() as u64;
        Ok(state.offset)
    }

    fn cut_off_segment(&self, state: &mut std::sync::MutexGuard<'_, SharedState>) {
              
        // Increment index - current_file_index is the biggest of all
        state.current_file_index += 1;
        let new_file_name = self.path.clone() + "/" + &state.current_file_index.to_string();

        // Create a new file
        let file = OpenOptions::new()
                        .read(true)
                        .append(true)
                        .create(true)
                        .open(new_file_name)
                        .expect("Failed to open file");

        // Add new file to file table
        let new_file_index = state.current_file_index;
        state.files.insert(new_file_index, file);
        state.offset = 0;        
    }
}

#[derive(Debug)]
pub enum KopperError {
    IO(io::Error),
    Parse(num::ParseIntError)
}

impl From<io::Error> for KopperError {
    fn from(err: io::Error) -> Self {
        KopperError::IO(err)
    }
}

impl From<num::ParseIntError> for KopperError {
    fn from(err: num::ParseIntError) -> Self {
        KopperError::Parse(err)
    }
}

impl SharedState {
    fn create(path: &str) -> Result<SharedState, KopperError> {
        let mut state = SharedState {
            table: HashMap::new(),
            files: BTreeMap::new(),
            offset: 0,
            current_file_index: 0,
        };

        // Create dir if doesn't exist yet
        match fs::create_dir(path) { _ => () };

        // Recover all files
        for dir_entry in fs::read_dir(path).unwrap() {

            let dir_entry = dir_entry.unwrap();

            let file = 
                OpenOptions::new()
                    .read(true)
                    .append(true)
                    .create(true)
                    .open(dir_entry.path())?;
            
            let file_index: FileIndex = 
                dir_entry.path()
                    .file_name().unwrap()
                    .to_str().unwrap()
                    .parse()?;

            println!("Recovering file: {}", file_index);

            state.files.insert(file_index, file);
            state.recover_file(file_index);
        }

        // If starting a new database, create the first file
        if state.files.is_empty() {
            let head_file = String::from(path) + "/" + &state.current_file_index.to_string();
            let file = OpenOptions::new()
                .read(true)
                .append(true)
                .create(true)
                .open(head_file)?;

            state.files.insert(0, file);
        }

        Ok(state)
    }

    fn recover_file(&mut self, file_index: FileIndex) {

        let mut file = self.files.get(&file_index).unwrap();

        enum CurrentlyReading { Key, Value }
        let mut currently_reading = CurrentlyReading::Key;
        let mut key = String::new();

        // With regards to current buffer
        let mut key_offset: usize;

        // With regards to file
        let mut value_file_offset: usize = 0; 
        let mut buffer_file_offset: usize = 0;
        
        let mut buffer = [0; 5];

        loop {
            let bytes_in_buffer = match file.read(&mut buffer) {
                Ok(bytes_read) if bytes_read == 0 => break,
                Ok(bytes_read) => bytes_read,
                Err(e) => {
                    println!("Error: {}", e); break
                }
            };
            key_offset = 0;
            
            for byte_index in 0..bytes_in_buffer {
                
                if buffer[byte_index] == b'\0' {

                    // TODO: if this is first byte: ERROR
                    
                    match currently_reading {
                        CurrentlyReading::Key => {
                            key.push_str(std::str::from_utf8(&buffer[key_offset..byte_index]).unwrap());
                            
                            value_file_offset = buffer_file_offset + byte_index + 1;
                            currently_reading = CurrentlyReading::Value;
                        },
                        CurrentlyReading::Value => {
                            // Swap pointers between key and empty string to avoid cloning
                            let mut tmp_key = String::new();
                            std::mem::swap(&mut tmp_key, &mut key);
                            
                            // Collected all needed parts: key, value's offset and length
                            self.table.insert(tmp_key, 
                                TableEntry {
                                    file_index,
                                    offset: value_file_offset as u64,
                                    len: buffer_file_offset + byte_index - value_file_offset,
                                });
                                
                            key_offset = byte_index + 1;
                            currently_reading = CurrentlyReading::Key;
                        }
                    }

                }
            }

            // Being here, we're probably left with some incomplete key or value that continues in the next chunk
            match currently_reading {
                CurrentlyReading::Key => {
                    key.push_str(std::str::from_utf8(&buffer[key_offset..bytes_in_buffer]).unwrap());
                },
                _ => ()
            }

            buffer_file_offset += bytes_in_buffer;
        }
    }
}