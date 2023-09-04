#[get("/")]
pub fn publish_message() -> &'static str {
    // to do: controller responsible for message publishing 
    "Hello, world!"
}

#[get("/offset")]
pub fn get_offset() -> &'static str {
    "offset"
}