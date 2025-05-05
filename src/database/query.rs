
#[derive(Clone)]
pub struct QueryOptions {
    pub decrypt: bool,
    pub aggregate_count: usize,
}

impl Default for QueryOptions {
    fn default() -> Self {
        Self {
            decrypt: true,
            aggregate_count: 1000,
        }
    }
}

impl QueryOptions {
    pub fn new(decrypt: bool, aggregate_count: usize) -> Self {
        Self { decrypt, aggregate_count }
    }
}
