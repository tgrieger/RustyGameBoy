pub struct Rom {
    content: Vec<u8>,
}

impl Rom {
    pub fn new(path: &str) -> std::io::Result<Rom> {
        Ok(Rom {
            content: std::fs::read(path)?,
        })
    }
}
