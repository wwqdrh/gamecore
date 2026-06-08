// GJson - Rust 等价实现 gamedb::GJson
// 提供路径查询的 JSON 文档存储，支持加密、文件持久化和变更订阅
// 路径格式使用 ";" 分隔符，例如 "init;player;health" 对应 data["init"]["player"]["health"]

use std::collections::HashMap;

use serde_json::Value;

type SubscribeCallback = Box<dyn Fn(&str, &Value) -> bool>;

pub struct FileStore {
    load_fn: Box<dyn Fn() -> Vec<u8>>,
    save_fn: Box<dyn Fn(Vec<u8>)>,
}

impl FileStore {
    pub fn new(
        load_fn: Box<dyn Fn() -> Vec<u8>>,
        save_fn: Box<dyn Fn(Vec<u8>)>,
    ) -> Self {
        Self { load_fn, save_fn }
    }

    pub fn load(&self) -> Vec<u8> {
        (self.load_fn)()
    }

    pub fn save(&self, data: Vec<u8>) {
        (self.save_fn)(data)
    }
}

pub struct GJson {
    data: Value,
    encrypted: bool,
    store: Option<FileStore>,
    subscribers: HashMap<String, Vec<SubscribeCallback>>,
}

const ENCRYPT_KEY: &[u8] = b"gamekit_gjson_encrypt_key_2026";

impl GJson {
    pub fn new(store: FileStore) -> Self {
        Self {
            data: Value::Object(serde_json::Map::new()),
            encrypted: false,
            store: Some(store),
            subscribers: HashMap::new(),
        }
    }

    pub fn enable_encrypt(&mut self) {
        self.encrypted = true;
    }

    pub fn disable_encrypt(&mut self) {
        self.encrypted = false;
    }

    pub fn load_by_store(&mut self) {
        if let Some(ref store) = self.store {
            let raw = store.load();
            if raw.is_empty() {
                return;
            }
            let json_str = if self.encrypted {
                Self::decrypt(&raw)
            } else {
                String::from_utf8_lossy(&raw).to_string()
            };
            if let Ok(val) = serde_json::from_str::<Value>(&json_str) {
                self.data = val;
            }
        }
    }

    pub fn encrypt(data: &str) -> Vec<u8> {
        let bytes = data.as_bytes();
        let key_len = ENCRYPT_KEY.len();
        bytes
            .iter()
            .enumerate()
            .map(|(i, &b)| b ^ ENCRYPT_KEY[i % key_len])
            .collect()
    }

    pub fn decrypt(data: &[u8]) -> String {
        let key_len = ENCRYPT_KEY.len();
        let decrypted: Vec<u8> = data
            .iter()
            .enumerate()
            .map(|(i, &b)| b ^ ENCRYPT_KEY[i % key_len])
            .collect();
        String::from_utf8_lossy(&decrypted).to_string()
    }

    pub fn update(&mut self, path: &str, action: &str, value: Value) {
        let parts: Vec<&str> = path.split(';').collect();
        if parts.is_empty() {
            return;
        }

        let should_set = if action == "~" {
            true
        } else {
            !self.has_path(&parts)
        };

        if should_set {
            self.set_path(&parts, value.clone());
            self.notify_subscribers(path, &value);
            self.save_by_store();
        }
    }

    pub fn subscribe(&mut self, path: &str, callback: SubscribeCallback) {
        if !self.subscribers.contains_key(path) {
            self.subscribers.insert(path.to_string(), Vec::new());
        }
        if let Some(callbacks) = self.subscribers.get_mut(path) {
            callbacks.push(callback);
        }
    }

    pub fn query(&self, path: &str) -> String {
        let parts: Vec<&str> = path.split(';').collect();
        let val = self.get_path(&parts);
        match val {
            Some(v) => v.to_string(),
            None => String::new(),
        }
    }

    pub fn query_value(&self, path: &str) -> Option<&Value> {
        let parts: Vec<&str> = path.split(';').collect();
        self.get_path(&parts)
    }

    pub fn to_value(json_str: &str) -> Value {
        serde_json::from_str(json_str).unwrap_or(Value::Null)
    }

    pub fn duplicate_all_string(&self) -> String {
        serde_json::to_string(&self.data).unwrap_or_default()
    }

    fn save_by_store(&self) {
        if let Some(ref store) = self.store {
            let json_str = serde_json::to_string(&self.data).unwrap_or_default();
            let raw = if self.encrypted {
                Self::encrypt(&json_str)
            } else {
                json_str.into_bytes()
            };
            store.save(raw);
        }
    }

    pub fn reload_data(&mut self, data: &str) {
        if let Ok(val) = serde_json::from_str::<Value>(data) {
            self.data = val;
        }
    }

    fn has_path(&self, parts: &[&str]) -> bool {
        let val = self.get_path(parts);
        match val {
            Some(v) => !v.is_null(),
            None => false,
        }
    }

    fn get_path(&self, parts: &[&str]) -> Option<&Value> {
        let mut current = &self.data;
        for part in parts {
            if part.is_empty() {
                continue;
            }
            match current {
                Value::Object(map) => {
                    current = map.get(*part)?;
                }
                Value::Array(arr) => {
                    if let Ok(idx) = part.parse::<usize>() {
                        current = arr.get(idx)?;
                    } else {
                        return None;
                    }
                }
                _ => return None,
            }
        }
        Some(current)
    }

    fn set_path(&mut self, parts: &[&str], value: Value) {
        if parts.is_empty() {
            return;
        }

        let len = parts.len();
        Self::set_path_recursive(&mut self.data, parts, 0, len, value);
    }

    fn set_path_recursive(
        data: &mut Value,
        parts: &[&str],
        depth: usize,
        len: usize,
        value: Value,
    ) {
        if depth >= len {
            return;
        }

        let part = parts[depth];
        if part.is_empty() {
            return;
        }

        if depth == len - 1 {
            if let Value::Object(ref mut map) = data {
                map.insert(part.to_string(), value);
            }
            return;
        }

        if let Value::Object(ref mut map) = data {
            if !map.contains_key(part) {
                map.insert(part.to_string(), Value::Object(serde_json::Map::new()));
            }
            if let Some(child) = map.get_mut(part) {
                Self::set_path_recursive(child, parts, depth + 1, len, value);
            }
        }
    }

    fn notify_subscribers(&self, path: &str, value: &Value) {
        if let Some(callbacks) = self.subscribers.get(path) {
            for callback in callbacks {
                if !callback(path, value) {
                    break;
                }
            }
        }
    }
}
