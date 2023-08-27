pub struct Copper {}

impl Copper {
    pub fn start(_path: &str) -> Self {
        Copper {}
    }

    pub fn read(&self, key: String) -> Option<String> {
        let mut ret = "Value for ".to_string();
        ret.push_str(&key);
        Some(ret)
    }

    pub fn write(&self, _key: String, _value: String) -> Option<()> {
        Some(())
    }
}