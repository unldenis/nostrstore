pub mod counter;
pub mod append_only;

/// A trait representing an operation that can be applied to a value.
/// This trait is used for events that can be applied to a value, such as incrementing or decrementing a counter.
pub trait Operation: Sized {
    type Value;

    fn default() -> Self::Value;
    fn deserialize(value: String) -> Result<Self, Box<dyn std::error::Error>>;
    fn serialize(&self) -> Result<String, Box<dyn std::error::Error>>;
    fn apply(&self, value: Self::Value) -> Self::Value;
}
