use std::time::Instant;
use std::collections::HashMap;

use crate::resp::Value;

struct StorageItem {
    value: String,
    create_time: Instant,
    expires: usize,
}

pub struct Storage {
    storage: HashMap<String, StorageItem>,
}

impl Storage {
    pub fn new() -> Self {
        Storage {
            storage: HashMap::new()
        }
    }
    
    pub fn set(&mut self, key: String, value: String, expires: usize) -> Value {
        let item = StorageItem {
            value: value,
            create_time: Instant::now(),
            expires: expires,
        };
        self.storage.insert(key, item);
        Value::SimpleString("OK".to_string())
    }

    pub fn get(&self, key: String) -> Value {
        let value = self.storage.get(key.as_str());
        match value {
            Some(v) => {
                match v.expires {
                    0 => {return Value::BulkString(v.value.clone());},
                    n => {
                        if v.create_time.elapsed().as_millis() > n as u128 { return Value::Null;}
                        else { return Value::BulkString(v.value.clone());}
                    }
                }
            },
            None => {return Value::Null;},
        }
    }
}

impl Default for Storage {
    fn default() -> Self {
        Storage::new()
    }
}