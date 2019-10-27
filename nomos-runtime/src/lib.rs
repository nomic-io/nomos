use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str;

extern "C" {
    fn print_str(ptr: *const u8, len: usize);

    // Allows this root state machine to upgrade in-place to a new WASM binary.
    // Calling `upgrade` will destroy all child instances.
    fn upgrade_code(code_ptr: *const u8, code_len: usize);
    fn execute_code(execute_msg_ptr: *const u8, execute_msg_len: usize);

    // Creates a child WASM instance using the specified binary.
    // Returns a remote reference to the WASM instance.
    // fn evaluate(ptr: *const u8, len: usize);

    // Persistent state interface
    fn get_length(length_result_ptr: *const usize, key_ptr: *const u8, key_len: usize);
    fn get_state(key_ptr: *const u8, key_len: usize, result_vec_ptr: *const u8);
    fn set_state(key_ptr: *const u8, key_len: usize, value_ptr: *const u8, value_len: usize);
}

pub fn print(msg: &str) {
    unsafe {
        print_str(msg.as_ptr(), msg.len());
    }
}

/**
 * Fetches the length of the value at the specified key.
 */
fn get_length_from_host(key: &[u8]) -> usize {
    let value_length: usize = 0;
    unsafe {
        let len_ptr: *const usize = &value_length;
        get_length(len_ptr, key.as_ptr(), key.len() as usize);
    }
    value_length
}

pub fn read<K: AsRef<[u8]>>(key: K) -> Option<Vec<u8>> {
    let value_length = get_length_from_host(key.as_ref());
    if value_length == 0 {
        return None;
    }
    print(&format!("value length: {}", value_length)[..]);
    // unsafe { get_state(key.as_ptr(), key.len() as usize) };
    let value_bytes = vec![0; value_length];

    // Tell the host to load up the value bytes for this key.
    unsafe {
        get_state(
            key.as_ref().as_ptr(),
            key.as_ref().len() as usize,
            value_bytes.as_ptr(),
        );
    }

    Some(value_bytes)
}

pub fn write<K: AsRef<[u8]>>(key: K, value: Vec<u8>) {
    unsafe {
        set_state(
            key.as_ref().as_ptr(),
            key.as_ref().len(),
            value.as_ptr(),
            value.len() as usize,
        )
    };
}

pub fn upgrade(code: Vec<u8>) {
    unsafe { upgrade_code(code.as_ptr(), code.len() as usize) };
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ExecutionMessage {
    pub code_key: Vec<u8>,
    pub store_key: Vec<u8>,
    pub entry_function_name: String,
}

pub fn execute(code_key: Vec<u8>, store_key: Vec<u8>, entry_function_name: &str) {
    let execute_msg = ExecutionMessage {
        code_key,
        store_key,
        entry_function_name: String::from(entry_function_name),
    };
    let execute_msg_bytes: Vec<u8> = bincode::serialize(&execute_msg).unwrap();
    unsafe {
        execute_code(execute_msg_bytes.as_ptr(), execute_msg_bytes.len() as usize);
    }
}

pub struct Child {
    code_key: Vec<u8>,
    store_key: Vec<u8>,
}

impl Child {
    pub fn new(code_key: Vec<u8>, store_key: Vec<u8>) -> Child {
        Child {
            code_key,
            store_key,
        }
    }

    pub fn get(&self, key: Vec<u8>) -> Option<Vec<u8>> {
        let current_store = read(&self.store_key);
        match current_store {
            Some(store_bytes) => {
                let store: HashMap<Vec<u8>, Vec<u8>> = bincode::deserialize(&store_bytes).unwrap();
                let value = store.get(&key);
                match value {
                    Some(val) => Some(val.to_vec()),
                    None => None,
                }
            }
            None => None,
        }
    }
    pub fn set(&self, key: Vec<u8>, value: Vec<u8>) {
        let current_store = read(&self.store_key);
        let mut child_store = match current_store {
            Some(store_bytes) => {
                let deserialized_store: HashMap<Vec<u8>, Vec<u8>> =
                    bincode::deserialize(&store_bytes[..]).unwrap();
                deserialized_store
            }
            None => {
                let child_store: HashMap<Vec<u8>, Vec<u8>> = HashMap::new();
                child_store
            }
        };

        child_store.insert(key, value);
        let serialized_store = bincode::serialize(&child_store).unwrap();
        write(&self.store_key, serialized_store);
    }
    pub fn call(&self, func_name: &str) {
        execute(self.code_key.clone(), self.store_key.clone(), func_name);
    }
}
