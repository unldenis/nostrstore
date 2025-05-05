pub mod counter;


pub trait Operation: Sized {
    type Value;

    fn default() -> Self::Value;
    fn deserialize(value: String) -> Result<Self, Box<dyn std::error::Error>>;
    fn serialize(&self) -> String;
    fn apply(&self, value: Self::Value) -> Self::Value;
}
