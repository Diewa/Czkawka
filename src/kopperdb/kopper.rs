use std::{collections::HashMap, sync::Mutex, fs::{File, OpenOptions}, io::Write, os::unix::prelude::FileExt };

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

        let file = OpenOptions::new()
                        .read(true)
                        .append(true)
                        .create(true)
                        .open(path)
                        .expect("Failed to open file");
        
        Ok(Kopper { 
            state: Mutex::new(SharedState { 
                table: HashMap::new(), 
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

    pub fn write(&self, key: String, value: String) -> std::io::Result<()> {
        
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
        state.offset = string_to_save.len();
        Ok(())
    }
}