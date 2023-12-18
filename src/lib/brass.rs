use std::{fs::{File, self, OpenOptions}, sync::{Mutex, Arc}, io::{self, Read, Seek, Write}};

const ROOT_NAME: &str = "0";

pub struct Brass {
    state: Arc<Mutex<SharedState>>,
    _path: String,
    segment_size: usize,
}

struct SharedState {
    root_file: File,
}

#[derive(Debug)]
pub enum BrassError {
    IO(io::Error)
}

impl From<io::Error> for BrassError {
    fn from(err: io::Error) -> Self {
        BrassError::IO(err)
    }
}

impl Brass {
    pub fn create(path: &str, segment_size: usize) -> Result<Self, BrassError> {

        // Create the DB directory if it doesn't exist
        match fs::create_dir_all(path) { _ => () };

        // If file exists - return it. If doesn't - create it.
        let mut file = 
        OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path.to_owned() + "/" + ROOT_NAME)?;

        if file.metadata().unwrap().len() == 0 {
            // This is a new database, create a full empty segment
            file.write(&vec![0; segment_size]).unwrap();

            // Mark the second byte of the segment with a tombstone - end of file symbol
            file.seek(io::SeekFrom::Start(1)).unwrap();
            file.write(&[b'\n']).unwrap();
        }

        Ok(Brass{ 
            segment_size, 
            _path: path.to_owned(), 
            state: Arc::new(Mutex::new(SharedState { root_file: file }))
        })
    }

    pub fn read(&self, key: &str) -> std::io::Result<Option<String>> {
        let mut state = self.state.lock().unwrap();
        let root = Segment::load(&mut state.root_file, self.segment_size);

        match root.iter() {
            SegmentIter::Leaf(iter) => {
                for (k, value, _) in iter {
                    if k == key {
                        return Ok(Some(value.to_owned()));
                    }
                }
                return Ok(None)
            },
            SegmentIter::Node(_) => {
                todo!()
            }
        }
    }
    pub fn write(&self, key: &str, value: &str) -> std::io::Result<usize> {
        
        // Load root into memory
        let mut state = self.state.lock().unwrap();
        let mut root = Segment::load(&mut state.root_file, self.segment_size);
        
        match root.iter() {
            SegmentIter::Leaf(_iter) => {
                if root.try_insert(&key, &value) {
                    state.root_file.rewind()?;
                    state.root_file.write(&root.buffer)?;
                    return Ok(key.len() + value.len());
                }

                todo!("Can't insert into the leaf, implement spliting!")
            },
            SegmentIter::Node(_) => {
                todo!()
            }
        }
    }
}

struct Segment {
    buffer: Vec<u8>
}

impl Segment {
    fn iter(&self) -> SegmentIter {
        SegmentIter::new(&self.buffer)
    }

    fn buf(&self) -> &[u8] {
        &self.buffer[1..]
    }

    fn buf_mut(&mut self) -> &mut [u8] {
        &mut self.buffer[1..]
    }

    fn try_insert(&mut self, key: &str, value: &str) -> bool {

        let mut offset;
        let end_offset;

        match self.iter() {
            
            SegmentIter::Leaf(iter) => {

                // Find the end of segment
                let mut tomb_index = None;
                for (index, byte) in self.buf().iter().enumerate() {
                    if byte == &b'\n' {
                        tomb_index = Some(index);
                        break;
                    }
                };
    
                // Segment is full to the brim
                if tomb_index.is_none() {
                    return false;
                }

                end_offset = tomb_index.unwrap();

                // Each entry is "key\0value\0", hence + 2
                let entry_size = key.len() + value.len() + 2;

                // Check if key and value would fit the buffer
                if entry_size + end_offset > self.buf().len() {
                    return false;
                }

                // Initialize offset to the last position in case the new key-value
                // should be put at the end
                offset = end_offset;

                // We can fit them. Go over existing entries to figure out where
                // - keys should be ordered alphabetically.
                for (k, _, o) in iter {
                    
                    if key < k {
                        // New key should be placed before the key 
                        // we are currently iterating over
                        offset = o;
                        break;
                    }
                }
            },
            
            SegmentIter::Node(_) => todo!(),
        }

        // TODO: tmp code
        if offset == end_offset {
            // Key
            self.buf_mut()[offset..(offset + key.len())].clone_from_slice(key.as_bytes());
            // Separator
            self.buf_mut()[offset + key.len()] = b'\0';
            // Value
            self.buf_mut()[(offset + key.len() + 1)..(offset + key.len() + 1 + value.len())].clone_from_slice(value.as_bytes());
            // Separator
            self.buf_mut()[offset + key.len() + 2 + value.len()] = b'\0';
            // Tombstone
            self.buf_mut()[offset + key.len() + 3 + value.len()] = b'\n';
            return true;
        }

        false
    }
}

enum SegmentIter<'a> {
    Leaf(LeafIterator<'a>),
    Node(NodeIterator)
}

struct LeafIterator<'a> {
    offset: usize,
    buffer: &'a [u8],
}

struct NodeIterator {
}

/// Iterator over key-value pairs stored in the segment.
/// Outputs tuple `(Key, Value, Index)``, where `Index` is 
/// the index of `Key` in the buffer with regards to `buf()`
/// 
impl<'a> Iterator for LeafIterator<'a> {

    type Item = (&'a str, &'a str, usize);
    
    fn next(&mut self) -> Option<Self::Item> {
        
        // Optimization: if data perfectly fills the file, we don't need tombstone
        if self.offset >= self.buffer.len() {
            return None;
        }

        // Tombstone - end of data in the segment
        if self.buffer[self.offset] == b'\n' {
            return None;
        }

        let mut key = "";
        
        // If offset of value is not set, we are reading key
        let mut value_offset = None;        

        // No end-of-data - start reading key
        for byte_index in self.offset..self.buffer.len() {
            
            if self.buffer[byte_index] == b'\0' {

                match value_offset {
                    // Key found
                    None => {
                        key = std::str::from_utf8(&self.buffer[self.offset..byte_index]).unwrap(); 
                        value_offset = Some(byte_index + 1);
                    },
                    // Value found
                    Some(offset) => {
                        let value = std::str::from_utf8(&self.buffer[offset..byte_index]).unwrap();
                        let ret = Some((key, value, self.offset));
                        self.offset += byte_index + 1;
                        return ret;
                    }
                }
            }
        }

        None
    }
}

impl<'a> SegmentIter<'a> {
    fn new(buf: &'a [u8]) -> Self {
        if buf[0] == 0b0 {
            return SegmentIter::Leaf(
                LeafIterator { 
                    offset: 0, 
                    buffer: &buf[1..]
                });
        }

        SegmentIter::Node(NodeIterator {})
    }
}

impl Segment {
    fn load(file: &mut File, segment_size: usize) -> Self {

        file.rewind().expect("Can't rewind file");

        let mut buffer = vec![];
        let bytes_read = file
            .read_to_end(&mut buffer)
            .expect("Can't read file");

        assert_eq!(bytes_read, segment_size);
        Segment { buffer }
    }
}


/// TESTS

#[test]
fn test_leaf_iterator_empty() {

    let segment = Segment { buffer: [b'\0', b'\n', 0, 0, 0].to_vec() };
    match segment.iter() {
        SegmentIter::Leaf(mut iter) => {
            assert_eq!(iter.next(), None);
        },
        _ => { panic!() },
    }
}

#[test]
fn test_leaf_iterator_one_value() {

    let segment = Segment { buffer: b"\0AB\0CD\0\n".to_vec() };
    let seg_iter =  segment.iter();
    match seg_iter {
        SegmentIter::Leaf(mut iter) => {
            assert_eq!(iter.next(), Some(("AB", "CD", 0)));
            assert_eq!(iter.next(), None);
        },
        _ => { panic!() },
    }
}

#[test]
fn test_leaf_iterator_no_tombstone() {

    let segment = Segment { buffer: b"\0AB\0CD\0".to_vec() };
    match segment.iter() {
        SegmentIter::Leaf(mut iter) => {
            assert_eq!(iter.next(), Some(("AB", "CD", 0)));
            assert_eq!(iter.next(), None);
        },
        _ => { panic!() },
    }
}