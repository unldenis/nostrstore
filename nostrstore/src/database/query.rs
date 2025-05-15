/// Query options for database queries.
/// This struct allows you to specify options for querying the database,
/// such as whether to decrypt the data and the maximum number of results to possibly aggregate.
/// It is used in the `read_history` method of the `Database` struct.
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
        Self {
            decrypt,
            aggregate_count,
        }
    }
}
