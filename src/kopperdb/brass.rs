pub struct Brass {
    
}

#[derive(Debug)]
pub enum BrassError {}

impl Brass {
    pub fn create(_path: &str, _segment_size: u64) -> Result<Self, BrassError> {
        Ok(Brass{})
    }

    pub fn read(&self, _key: String) -> std::io::Result<Option<String>> {
        Ok(Some(String::new()))
    }
    pub fn write(&self, _key: String, _value: String) -> std::io::Result<u64> {
        Ok(0)
    }
}