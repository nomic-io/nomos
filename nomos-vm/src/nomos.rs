use byteorder::ByteOrder;
use failure::Error;
use nomos_runtime::ExecutionMessage;
use serde::{Deserialize, Serialize};
use std::boxed::Box;
use std::collections::HashMap;
use std::str;
use wasmer_runtime::{
    error, func, imports, instantiate, units, Ctx, ImportObject, Instance, Value,
};

pub struct VM {
    code: Vec<u8>,
    pub state: HashMap<Vec<u8>, Vec<u8>>,
}

type Instantiator = fn(&[u8], &ImportObject) -> error::Result<Instance>;
pub struct SharedContext {
    pub state: HashMap<Vec<u8>, Vec<u8>>,
    pub code: Vec<u8>,
}

impl VM {
    pub fn new(code: Vec<u8>) -> VM {
        let mut state: HashMap<Vec<u8>, Vec<u8>> = HashMap::new();
        VM { code, state }
    }

    pub fn set<K: AsRef<[u8]>, V: AsRef<[u8]>>(&mut self, key: K, value: V) {
        self.state
            .insert(key.as_ref().to_vec(), value.as_ref().to_vec());
    }

    pub fn get<K: AsRef<[u8]>>(&self, key: K) -> Option<&Vec<u8>> {
        self.state.get(&key.as_ref().to_vec())
    }

    pub fn call(&mut self, function_name: &str) {
        let mut last_state: HashMap<Vec<u8>, Vec<u8>> = self.state.clone();

        let mut shared_context = SharedContext {
            state: last_state,
            code: self.code.clone(),
        };

        let get_length = |ctx: &mut Ctx, length_result_ptr: u32, key_ptr: u32, key_len: u32| {
            // Think of this line as a very fancy reference to the state variable from above:
            // let mut state: &mut HashMap<Vec<u8>, Vec<u8>> =
            //     unsafe { &mut *(ctx.data as *mut HashMap<Vec<u8>, Vec<u8>>) };
            let mut shared_context: &mut SharedContext =
                unsafe { &mut *(ctx.data as *mut SharedContext) };
            let state = &mut shared_context.state;
            let memory = ctx.memory(0);
            let key_vec: Vec<_> = memory.view()[key_ptr as usize..(key_ptr + key_len) as usize]
                .iter()
                .map(|cell: &std::cell::Cell<u8>| cell.get())
                .collect();
            let state_value = state.get(&key_vec);
            match state_value {
                Some(val) => {
                    let mut value_length_to_write = [0 as u8; 4];
                    byteorder::LittleEndian::write_u32(
                        &mut value_length_to_write,
                        val.len() as u32,
                    );
                    // Next, write the length value bytes into wasm memory
                    for (byte, cell) in value_length_to_write.iter().zip(
                        memory.view()[length_result_ptr as usize..(length_result_ptr + 4) as usize]
                            .iter(),
                    ) {
                        cell.set(*byte);
                    }
                }
                None => (),
            };
        };

        let get_state = |ctx: &mut Ctx, key_ptr: u32, key_len: u32, result_vec_ptr: u32| {
            // Think of this line as a very fancy reference to the state variable from above:
            let mut shared_context: &mut SharedContext =
                unsafe { &mut *(ctx.data as *mut SharedContext) };
            let state = &mut shared_context.state;

            let memory = ctx.memory(0);
            let key_vec: Vec<_> = memory.view()[key_ptr as usize..(key_ptr + key_len) as usize]
                .iter()
                .map(|cell: &std::cell::Cell<u8>| cell.get())
                .collect();

            let state_value = state.get(&key_vec).unwrap();
            // Copy state into memory
            for (byte, cell) in state_value.iter().zip(
                memory.view()
                    [result_vec_ptr as usize..result_vec_ptr as usize + state_value.len() as usize]
                    .iter(),
            ) {
                cell.set(*byte);
            }
        };

        let set_state =
            |ctx: &mut Ctx, key_ptr: u32, key_len: u32, value_ptr: u32, value_len: u32| {
                let mut shared_context: &mut SharedContext =
                    unsafe { &mut *(ctx.data as *mut SharedContext) };
                let state = &mut shared_context.state;

                let memory = ctx.memory(0);
                let key_vec: Vec<_> = memory.view()[key_ptr as usize..(key_ptr + key_len) as usize]
                    .iter()
                    .map(|cell: &std::cell::Cell<u8>| cell.get())
                    .collect();

                let value_vec: Vec<_> = memory.view()
                    [value_ptr as usize..(value_ptr + value_len) as usize]
                    .iter()
                    .map(|cell: &std::cell::Cell<u8>| cell.get())
                    .collect();

                state.insert(key_vec, value_vec);
            };

        let upgrade_code = |ctx: &mut Ctx, code_ptr: u32, code_len: u32| {
            let mut shared_context: &mut SharedContext =
                unsafe { &mut *(ctx.data as *mut SharedContext) };
            let state = &mut shared_context.state;
            let memory = ctx.memory(0);
            let code_vec: Vec<_> = memory.view()[code_ptr as usize..(code_ptr + code_len) as usize]
                .iter()
                .map(|cell: &std::cell::Cell<u8>| cell.get())
                .collect();

            shared_context.code = code_vec;
        };

        let execute_code = |ctx: &mut Ctx,
                            execution_msg_bytes_ptr: u32,
                            execution_msg_bytes_len: u32| {
            let mut shared_context: &mut SharedContext =
                unsafe { &mut *(ctx.data as *mut SharedContext) };
            let state = &mut shared_context.state;
            let memory = ctx.memory(0);
            let execution_msg_bytes: Vec<_> = memory.view()[execution_msg_bytes_ptr as usize
                ..(execution_msg_bytes_ptr + execution_msg_bytes_len) as usize]
                .iter()
                .map(|cell: &std::cell::Cell<u8>| cell.get())
                .collect();

            let execution_msg: ExecutionMessage =
                bincode::deserialize(&execution_msg_bytes).unwrap();

            let code_to_execute = state.get(&execution_msg.code_key);
            match code_to_execute {
                Some(code) => {
                    let mut child_vm = VM::new(code.to_vec());
                    let value_at_store_key = state.get(&execution_msg.store_key);
                    if let Some(value_bytes) = value_at_store_key {
                        let child_store_result: Result<
                            HashMap<Vec<u8>, Vec<u8>>,
                            Box<bincode::ErrorKind>,
                        > = bincode::deserialize(&value_bytes);
                        match child_store_result {
                            Err(e) => {
                                // Return execution error: invalid store bytes
                                return ();
                            }
                            Ok(child_store) => {
                                child_vm.state = child_store;
                                let execution_result =
                                    child_vm.call(&execution_msg.entry_function_name[..]);
                                // TODO: Handle execution result
                                let child_store_bytes =
                                    bincode::serialize(&child_vm.state).unwrap();
                                state.insert(execution_msg.store_key.to_vec(), child_store_bytes);
                            }
                        }
                    } else {
                        // Execution error result: store not found
                        return ();
                    };
                }
                None => {
                    // Return execution result with error: code not found
                }
            };
        };

        let dtor = (|_: *mut std::ffi::c_void| {}) as fn(*mut std::ffi::c_void);
        let ptr = &mut shared_context as *mut _ as *mut std::ffi::c_void;
        let import_object = imports! {
            move || {
                (ptr, dtor)
            },
            "env" => {
                "print_str" => func!(print_str),
                "get_state" => func!(get_state),
                "set_state" => func!(set_state),
                "get_length" => func!(get_length),
                "upgrade_code" => func!(upgrade_code),
                "execute_code" => func!(execute_code),
            },
        };

        let mut instance = instantiate(self.code.as_slice(), &import_object).unwrap();
        // Write action bytes into wasm memory
        let memory = instance.context_mut().memory(0);

        instance
            .call(function_name, &[])
            .expect("Calling vm method failed");

        self.state = shared_context.state.clone();
        self.code = shared_context.code.clone();
    }
}

fn print_str(ctx: &mut Ctx, ptr: u32, len: u32) {
    // Get a slice that maps to the memory currently used by the webassembly
    // instance.
    //
    // Webassembly only supports a single memory for now,
    // but in the near future, it'll support multiple.
    //
    // Therefore, we don't assume you always just want to access first
    // memory and force you to specify the first memory.
    let memory = ctx.memory(0);

    // Get a subslice that corresponds to the memory used by the string.
    let str_vec: Vec<_> = memory.view()[ptr as usize..(ptr + len) as usize]
        .iter()
        .map(|cell| cell.get())
        .collect();

    // Convert the subslice to a `&str`.
    let string = str::from_utf8(&str_vec).unwrap();

    // Print it!
    println!("{}", string);
}
