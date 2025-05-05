
use nostr_sdk::prelude::*;

//// Represents a trait for operations that can be performed on a value
/// in the Nostr database. This trait defines methods for
/// serializing, deserializing,
/// and applying operations to values.
pub trait Operation: Sized {
    type Value;

    fn default() -> Self::Value;

    fn deserialize(value: String) -> Result<Self, Box<dyn std::error::Error>>;

    fn serialize(&self) -> String;

    fn apply(&self, value: Self::Value) -> Self::Value;
}


pub enum CounterEvent  {
    Increment,
    Decrement,
}

impl Operation for CounterEvent {
    type Value = i64;

    fn default() -> i64 {
        0
    }

    fn deserialize(value: String) -> Result<CounterEvent, Box<dyn std::error::Error>> {
        Ok(match value.as_str() {
            "increment" => CounterEvent::Increment,
            "decrement" => CounterEvent::Decrement,
            _ => {
                return Err(format!("Invalid operation: {}", value).into());
            }
        })
    }

    fn serialize(&self) -> String {
        match self {
            CounterEvent::Increment => "increment".to_string(),
            CounterEvent::Decrement => "decrement".to_string(),
        }
    }


    fn apply(&self, value: i64) -> i64 {
        match self {
            CounterEvent::Increment => value + 1,
            CounterEvent::Decrement => value - 1,
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



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_counter_full_flow() {
        let input_ops = vec!["increment", "decrement"];
        let mut value = CounterEvent::default();

        for input in input_ops {
            let op = CounterEvent::deserialize(input.to_string()).unwrap();
            value = op.apply(value);// The `Operation` trait is used to define operations that can be performed

        }

        // Flow: 0 + 1 - 1 → 
        assert_eq!(value, 0);
    }

    #[test]
    fn test_payment_flow_mixed_statuses() {
        let ops = vec![
            "100,pending",
            "200,paid",
            "50,pending",
        ];

        let mut paid_status = PaymentStatus::default();

        for op_str in ops {
            let op = PaymentStatus::deserialize(op_str.to_string()).unwrap();
            paid_status = op.apply(paid_status);
        }

        // Only one "paid" → final result should be true
        assert_eq!(paid_status, true);
    }

    #[test]
    fn test_payment_serialize_deserialize_roundtrip() {
        let original = PaymentStatus {
            amount: 150,
            status: "paid".into(),
        };

        let serialized = original.serialize();
        let deserialized = PaymentStatus::deserialize(serialized).unwrap();

        assert_eq!(deserialized.amount, 150);
        assert_eq!(deserialized.status, "paid");
    }
}

