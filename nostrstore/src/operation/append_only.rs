use serde::{de::DeserializeOwned, Deserialize, Serialize};

use super::Operation;


pub struct Person {
    pub name: String,
    pub age: u8,
}


pub struct AppendOnlyEvent<T : Clone + Serialize + DeserializeOwned> {
    pub value: T,
}

impl <T : Clone + Serialize + DeserializeOwned> AppendOnlyEvent<T> {
    pub fn new(value: T) -> Self {
        Self { value }
    }
}

impl <T : Clone + Serialize + DeserializeOwned> Operation for AppendOnlyEvent<T> {
    type Value = Vec<T>;

    fn default() -> Self::Value {
        Vec::new()
    }

    fn deserialize(value: String) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self { value: serde_json::from_str(&value)? })
    }

    fn serialize(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string(&self.value)?)
    }

    fn apply(&self, mut value: Self::Value) -> Self::Value {
        value.push(self.value.clone());
        value
    }
}