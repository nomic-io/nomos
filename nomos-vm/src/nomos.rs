use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str;
use wasmer_runtime::{error, func, imports, instantiate, units, Ctx, Value};

pub struct VM {
    code: Vec<u8>,
    pub state: HashMap<Vec<u8>, Vec<u8>>,
}

impl VM {
    pub fn new(code: Vec<u8>) -> VM {
        let mut state: HashMap<Vec<u8>, Vec<u8>> = HashMap::new();
        let initial_count = 0;
        state.insert(
            b"count".to_vec(),
            bincode::serialize(&initial_count).unwrap(),
        );
        VM { code, state }
    }

    pub fn next(&mut self, action_bytes: &Vec<u8>) {
        let mut state: HashMap<Vec<u8>, Vec<u8>> = self.state.clone();
        state.insert(b"input".to_vec(), action_bytes.to_vec());
        state.insert(b"code".to_vec(), self.code.to_vec());

        let get_length = |ctx: &mut Ctx, length_result_ptr: u32, key_ptr: u32, key_len: u32| {
            // Think of this line as a very fancy reference to the state variable from above:
            let mut state: &mut HashMap<Vec<u8>, Vec<u8>> =
                unsafe { &mut *(ctx.data as *mut HashMap<Vec<u8>, Vec<u8>>) };

            let memory = ctx.memory(0);
            let key_vec: Vec<_> = memory.view()[key_ptr as usize..(key_ptr + key_len) as usize]
                .iter()
                .map(|cell: &std::cell::Cell<u8>| cell.get())
                .collect();
            let state_value = state.get(&key_vec).unwrap();
            let mut value_length_to_write = [0 as u8; 4];
            byteorder::LittleEndian::write_u32(
                &mut value_length_to_write,
                state_value.len() as u32,
            );
            // Next, write the length value bytes into wasm memory
            for (byte, cell) in value_length_to_write.iter().zip(
                memory.view()[length_result_ptr as usize..(length_result_ptr + 4) as usize].iter(),
            ) {
                cell.set(*byte);
            }
        };

        let get_state = |ctx: &mut Ctx, key_ptr: u32, key_len: u32, result_vec_ptr: u32| {
            // Think of this line as a very fancy reference to the state variable from above:
            let mut state: &mut HashMap<Vec<u8>, Vec<u8>> =
                unsafe { &mut *(ctx.data as *mut HashMap<Vec<u8>, Vec<u8>>) };

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
                let mut state: &mut HashMap<Vec<u8>, Vec<u8>> =
                    unsafe { &mut *(ctx.data as *mut HashMap<Vec<u8>, Vec<u8>>) };

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
            let mut state: &mut HashMap<Vec<u8>, Vec<u8>> =
                unsafe { &mut *(ctx.data as *mut HashMap<Vec<u8>, Vec<u8>>) };
            let memory = ctx.memory(0);
            let code_vec: Vec<_> = memory.view()[code_ptr as usize..(code_ptr + code_len) as usize]
                .iter()
                .map(|cell: &std::cell::Cell<u8>| cell.get())
                .collect();

            state.insert(b"code".to_vec(), code_vec);
        };

        let dtor = (|_: *mut std::ffi::c_void| {}) as fn(*mut std::ffi::c_void);
        let ptr = &mut state as *mut _ as *mut std::ffi::c_void;
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
            },
        };

        let mut instance = instantiate(self.code.as_slice(), &import_object).unwrap();
        // Write action bytes into wasm memory
        let memory = instance.context_mut().memory(0);
        // TODO: perhaps grow memory depending on input size?

        // let memory_grow_result = memory.grow(units::Pages(10));
        // println!("memory grow result: {:?}", memory_grow_result);
        // for (byte, cell) in action_bytes
        //     .iter()
        //     .zip(memory.view()[0 as usize..action_bytes.len() as usize].iter())
        // {
        //     cell.set(*byte);
        // }

        let values = instance
            .call(
                "_run",
                // &[Value::I32(0), Value::I32(action_bytes.len() as i32)],
                &[],
            )
            .expect("Calling run method failed");

        self.state = state.clone();
        self.code = state.get(&b"code".to_vec()).unwrap().to_vec();
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
