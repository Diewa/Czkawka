/// This example aims to present how to serialize and deserialize
/// files with serde and it's "bincode" extension
///
/// Requirements:
/// 1. Serialize structs one by one with as little overhead as possible
/// 2. Load a file with multiple subsequent serialized structs to memory, 
///    and deserialize them iteratively

fn serde_with_bincode_and_iterate() {

    // bincode library adds encoding to serde library that's the closest to 
    // in-memory representation of data

    // Step 1. Derive our struct from serde 
    #[derive(Debug, serde::Serialize, serde::Deserialize)]
    struct MyStruct {
        name: String,
        age: u32,
    }

    // Create few instances of our struct 
    let o1 = MyStruct { name: String::from("A"), age: 25 };
    let o2 = MyStruct { name: String::from("B"), age: 26 };
    let o3 = MyStruct { name: String::from("C"), age: 27 };

    // Serialize all into the same Vec<u8> (one after another)
    let mut serialized_data: Vec<u8> = vec![];
    serialized_data.append(&mut bincode::serialize(&o1).unwrap());
    serialized_data.append(&mut bincode::serialize(&o2).unwrap());
    serialized_data.append(&mut bincode::serialize(&o3).unwrap());

    // Now how to deserialize it?
    // We could use `bincode::deserialize_from(R)` where R is a reader that implements std::io::Read
    // There is a structure like that called std::io::Cursor<T> which takes ownership of some [u8] 
    // and keeps a pointer to the last read address. After each read the address is updated,
    // so it's just like reading a file!
    // example: 
    // let cursor = Cursor::new(serialized_data); 
    // let deserialized: MyStruct = bincode::deserialize_from(cursor).unwrap();
    //
    // However, theres one problem: deserialize_from consumes the cursor! What a bad design.
    // We can fix it by wrapping the cursor into our own struct, where it would be kept inside Rc<RefCell>
    // That way we can clone it into each deserialized_from() and each would update the "cursor"
    // We'd have to also make it implement std::io::Read. Let's try
    use std::rc::Rc;
    use std::cell::RefCell;
    use std::io::Cursor;

    // MyCursor type holding an std::io::Cursor, and implementing Clone
    #[derive(Clone)]
    struct MyCursor<T> {
        cursor: Rc<RefCell<Cursor<T>>>,
    }
    
    // std::io::Read implementation for our wrapper
    impl<T: AsRef<[u8]>> std::io::Read for MyCursor<T> {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {

            // Pass to internal cursor, RefCell requires callee to use borrow_mut() for each mutation
            self.cursor.borrow_mut().read(buf)
        }
    }

    // Instantiate our cursor
    let cursor = MyCursor { 
        cursor: Rc::new(RefCell::new(
            Cursor::new(serialized_data)
        )) 
    };

    // Deserialize the struct from the Read trait object
    let d1: MyStruct = bincode::deserialize_from(cursor.clone()).unwrap();
    let d2: MyStruct = bincode::deserialize_from(cursor.clone()).unwrap();
    let d3: MyStruct = bincode::deserialize_from(cursor.clone()).unwrap();

    // Print the deserialized struct
    println!("{:?}, {:?}, {:?}", d1, d2, d3);

}

fn main() {
    serde_with_bincode_and_iterate();
}