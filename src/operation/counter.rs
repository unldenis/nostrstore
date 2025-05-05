use super::Operation;

pub enum CounterEvent {
    Increment,
    Decrement,
}

impl Operation for CounterEvent {
    type Value = i64;

    fn default() -> i64 {
        0
    }

    fn deserialize(value: String) -> Result<Self, Box<dyn std::error::Error>> {
        match value.as_str() {
            "increment" => Ok(Self::Increment),
            "decrement" => Ok(Self::Decrement),
            _ => Err(format!("Invalid operation: {}", value).into()),
        }
    }

    fn serialize(&self) -> String {
        match self {
            Self::Increment => "increment".to_string(),
            Self::Decrement => "decrement".to_string(),
        }
    }

    fn apply(&self, value: i64) -> i64 {
        match self {
            Self::Increment => value + 1,
            Self::Decrement => value - 1,
        }
    }
}
