
use std::default;

use nostr_sdk::prelude::*;
use crate::{db::QueryOptions, Database, NostrDBError};

pub trait Operation<T> : Sized {
    fn default() -> T;

    fn deserialize(value: String) -> Result<Self, Box<dyn std::error::Error>>;

    fn serialize(&self) -> String;

    fn apply(&self, value: T) -> T;

    async fn store<I : Into<String>>(&self, db : &Database, key : I) -> Result<EventId, NostrDBError> {
        let serialized = self.serialize();
        db.store(key, &serialized).await
    }

    async fn read<I : Into<String>>(db : &Database, key : I) -> Result<T, NostrDBError> {
        let values = db.read(key, QueryOptions::new(true)).await?;

        let mut default = Self::default();

        for ele in values {
            let operation = Self::deserialize(ele.value).map_err(|e|NostrDBError::EventStreamError(e.to_string()))?;
            default = operation.apply(default);
        }
        Ok(default)
    }
}


pub enum CounterExample  {
    Increment,
    Decrement,
    Set(i64),
}

impl Operation<i64> for CounterExample {

    fn default() -> i64 {
        0
    }

    fn deserialize(value: String) -> Result<CounterExample, Box<dyn std::error::Error>> {
        Ok(match value.as_str() {
            "increment" => CounterExample::Increment,
            "decrement" => CounterExample::Decrement,
            _ => CounterExample::Set(value.parse::<i64>().unwrap()),
        })
    }

    fn serialize(&self) -> String {
        match self {
            CounterExample::Increment => "increment".to_string(),
            CounterExample::Decrement => "decrement".to_string(),
            CounterExample::Set(value) => value.to_string(),
        }
    }


    fn apply(&self, value: i64) -> i64 {
        match self {
            CounterExample::Increment => value + 1,
            CounterExample::Decrement => value - 1,
            CounterExample::Set(new_value) => *new_value,
        }
    }
}



pub struct PaymentStatus {
    pub amount: i64,
    pub status: String,
}

impl Operation<bool> for PaymentStatus {
    fn default() -> bool {
        false
    }

    fn deserialize(value: String) -> Result<PaymentStatus, Box<dyn std::error::Error>> {
        let parts: Vec<&str> = value.split(',').collect();
        let amount = parts[0].parse::<i64>().unwrap();
        let status = parts[1].to_string();
        Ok(PaymentStatus { amount, status })
    }

    fn serialize(&self) -> String {
        format!("{},{}", self.amount, self.status)
    }

    fn apply(&self, value: bool) -> bool {
        if self.status == "paid" {
            true
        } else {
            value
        }
    }
}




