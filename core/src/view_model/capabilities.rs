pub trait Capabilities {
    /// returns None if file not bakefile exist
    fn read_file(&self, file_name: &str) -> Option<String>;
}
