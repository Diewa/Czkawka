use std::{collections::HashMap, sync::Mutex, fs::{File, OpenOptions}, io::{Write, Read}, os::unix::prelude::FileExt };

pub struct Kopper {
    state: Mutex<SharedState>,
}

struct SharedState {
    table: HashMap<String, TableEntry>,
    offset: usize,
    file: File
}

struct TableEntry {
    offset: u64,
    len: usize
}


impl Kopper {
    pub fn start(path: &str) -> Result<Self, std::io::Error> {

        let mut file = OpenOptions::new()
                        .read(true)
                        .append(true)
                        .create(true)
                        .open(path)
                        .expect("Failed to open file");

        // Recover
        let mut table = HashMap::new();
        if file.metadata().unwrap().len() != 0 {
            table = Kopper::recover(&mut file);
        }

        Ok(Kopper { 
            state: Mutex::new(SharedState { 
                table, 
                offset: 0, 
                file 
            })
        })
    }

    pub fn read(&self, key: String) -> std::io::Result<Option<String>> {
        let state = self.state.lock().unwrap();

        let table_entry = match state.table.get(&key) {
            Some(table_entry) => table_entry,
            None => return Ok(None),
        };

        let mut buffer = vec![0; table_entry.len];
        state.file.read_exact_at(buffer.as_mut(), table_entry.offset)?;

        Ok(Some(
            String::from_utf8(buffer)
                .expect("Can't deserialize buffer to string")
        ))
    }

    pub fn write(&self, key: String, value: String) -> std::io::Result<usize> {
        
        let mut state = self.state.lock().unwrap();

        // 1. Save in in-memory map
        let entry = TableEntry {
            offset: key.as_bytes().len() as u64 + 1,
            len: value.as_bytes().len()
        };

        
        state.table.insert(key.clone(), entry);

        // 2. Write to disk
        let mut string_to_save = key;
        string_to_save.push('\0');
        string_to_save.push_str(&value);
        string_to_save.push('\0');
        
        let string_to_save = string_to_save.as_bytes();
        state.file.write_all(string_to_save)?;

        // Update current offset 
        state.offset += string_to_save.len();
        Ok(state.offset)
    }

    fn recover(file: &mut File) -> HashMap<String, TableEntry> {
        enum CurrentlyReading { Key, Value }
        let mut currently_reading = CurrentlyReading::Key;
        let mut key = String::new();

        // With regards to current buffer
        let mut key_offset: usize;

        // With regards to file
        let mut value_file_offset: usize = 0; 
        let mut buffer_file_offset: usize = 0;
        
        let mut table = HashMap::new();
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
                            // Swap pointers betwen key and empty string to avoid cloning
                            let mut tmp_key = String::new();
                            std::mem::swap(&mut tmp_key, &mut key);
                            
                            // Collected all needed parts: key, value's offset and length
                            table.insert(tmp_key, 
                                TableEntry {
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

        table
    }
}