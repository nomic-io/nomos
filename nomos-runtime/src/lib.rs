pub mod root {
    use byteorder::ByteOrder;
    use serde::{Deserialize, Serialize};
    use std::slice;
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

    pub fn read(key: &[u8]) -> Vec<u8> {
        let value_length = get_length_from_host(key);
        // unsafe { get_state(key.as_ptr(), key.len() as usize) };
        let value_bytes = vec![0; value_length];

        // Tell the host to load up the value bytes for this key.
        unsafe {
            get_state(key.as_ptr(), key.len() as usize, value_bytes.as_ptr());
        }

        value_bytes
    }

    pub fn write(key: &[u8], value: Vec<u8>) {
        unsafe {
            set_state(
                key.as_ptr(),
                key.len(),
                value.as_ptr(),
                value.len() as usize,
            )
        };
    }

    pub fn upgrade(code: Vec<u8>) {
        unsafe { upgrade_code(code.as_ptr(), code.len() as usize) };
    }

    pub struct ExecutionContext {
        limit: u32,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub enum ExecutionMessage {
        Start { code_key: Vec<u8> },
    }

    pub fn execute(code_key: Vec<u8>) {
        let get_four = || {
            print("Called get_four");
            4
        };
        let execute_msg = ExecutionMessage::Start { code_key };
        let execute_msg_bytes: Vec<u8> = bincode::serialize(&execute_msg).unwrap();
        unsafe {
            execute_code(execute_msg_bytes.as_ptr(), execute_msg_bytes.len() as usize);
        }
    }
}

pub mod child {
    extern "C" {
        fn print_str(ptr: *const u8, len: usize);
    }

    pub fn print(msg: &str) {
        unsafe {
            print_str(msg.as_ptr(), msg.len());
        }
    }
}
