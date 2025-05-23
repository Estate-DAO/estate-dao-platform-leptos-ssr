// sort_json.rs
use serde_json::{Map, Value};

/// Recursively sorts JSON objects by their keys.
/// Arrays are traversed, but the order of array elements remains the same.
/// Each element of the array is also sorted if it is a nested object/array.
pub fn sort_json(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut sorted_map = Map::new();
            // Collect keys and sort them alphabetically
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            // For each key in sorted order, recursively sort its value
            for &k in &keys {
                sorted_map.insert(k.clone(), sort_json(&map[k]));
            }
            Value::Object(sorted_map)
        }
        Value::Array(arr) => {
            // Sort each element of the array (in case elements are objects/arrays)
            Value::Array(arr.iter().map(sort_json).collect())
        }
        // Primitives (String, Number, Bool, Null) remain as is
        other => other.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_sort_json_simple_object() {
        let input = json!({
            "c": 3,
            "a": 1,
            "b": 2
        });
        let expected = json!({
            "a": 1,
            "b": 2,
            "c": 3
        });

        assert_eq!(sort_json(&input), expected);
    }

    #[test]
    fn test_sort_json_nested_object_and_arrays() {
        let input = json!({
            "z": {
                "y": 2,
                "x": [ {"b": 2, "a": 1}, {"d": 4, "c": 3} ],
            },
            "a": 1
        });
        let expected = json!({
            "a": 1,
            "z": {
                "x": [
                    {"a": 1, "b": 2},
                    {"c": 3, "d": 4}
                ],
                "y": 2
            }
        });

        assert_eq!(sort_json(&input), expected);
    }

    #[test]
    fn test_sort_json_primitives() {
        let input = json!(42);
        let expected = json!(42);

        // Primitives should remain the same
        assert_eq!(sort_json(&input), expected);
    }

    #[test]
    fn test_json_compact_serialization() {
        let input = json!({
            "z": {
                "y":    2,  // Extra spaces
                "x": [
                    {
                        "b": 2,
                        "a": 1
                    },
                    {
                        "d": 4,
                        "c": 3
                    }
                ]
            },
            "a": 1
        });

        // First sort the JSON
        let sorted = sort_json(&input);

        // Then serialize to compact string
        let compact = serde_json::to_string(&sorted).unwrap();

        // Expected compact form (no whitespace)
        let expected = r#"{"a":1,"z":{"x":[{"a":1,"b":2},{"c":3,"d":4}],"y":2}}"#;

        assert_eq!(compact, expected);
    }

    #[test]
    fn test_nowpayments_example() {
        // Example from NowPayments documentation
        let input = json!({
            "payment_id": 5524759,
            "payment_status": "finished",
            "pay_address": "0x123...",
            "price_amount": 3999.5,
            "price_currency": "usd",
            "order_id": "ABC123"
        });

        let sorted = sort_json(&input);
        let compact = serde_json::to_string(&sorted).unwrap();

        // Verify keys are sorted and no whitespace
        let expected = r#"{"order_id":"ABC123","pay_address":"0x123...","payment_id":5524759,"payment_status":"finished","price_amount":3999.5,"price_currency":"usd"}"#;

        assert_eq!(compact, expected);
    }
}
