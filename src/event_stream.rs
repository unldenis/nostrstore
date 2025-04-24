
use std::default;

use nostr_sdk::prelude::*;
use crate::{db::QueryOptions, Database, NostrDBError};

pub trait Operation: Sized {
    type Value;

    fn default() -> Self::Value;

    fn deserialize(value: String) -> Result<Self, Box<dyn std::error::Error>>;

    fn serialize(&self) -> String;

    fn apply(&self, value: Self::Value) -> Self::Value;
}


pub enum CounterExample  {
    Increment,
    Decrement,
    Set(i64),
}

impl Operation for CounterExample {
    type Value = i64;

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

impl Operation for PaymentStatus {
    type Value = bool;

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




