// use byteorder::ByteOrder;
// use std::slice;
// use std::str;

// extern "C" {
//     fn print_str(ptr: *const u8, len: usize);

//     // Allows this root state machine to upgrade in-place to a new WASM binary.
//     // Calling `upgrade` will destroy all child instances.
//     // fn upgrade();

//     // Creates a child WASM instance using the specified binary.
//     // Returns a remote reference to the WASM instance.
//     // fn evaluate(ptr: *const u8, len: usize);

//     // Persistent state interface
//     fn get_state(ptr: *const u8, len: usize);
//     fn set_state(key_ptr: *const u8, key_len: usize, value_ptr: *const u8, value_len: usize);

// }

// pub fn print(msg: &str) {
//     unsafe {
//         print_str(msg.as_ptr(), msg.len());
//     }
// }

// pub fn get_state_from_host(key: &[u8]) -> Vec<u8> {
//     // print("asking host for state..");
//     unsafe { get_state(key.as_ptr(), key.len() as usize) };
//     // print("got state from host");

//     let value_len_bytes: &[u8] = unsafe { slice::from_raw_parts(0 as _, 4 as _) };

//     // Ask Matt: Why does this program fail if I don't do this format thing?
//     &format!("value len bytes: {:?}", value_len_bytes)[..];
//     let value_len = byteorder::BigEndian::read_i32(value_len_bytes);

//     let value_bytes = unsafe { slice::from_raw_parts(4 as _, value_len as _) };
//     value_bytes.to_vec()
// }

// pub fn set_state_on_host(key: &[u8], value: Vec<u8>) {
//     unsafe {
//         set_state(
//             key.as_ptr(),
//             key.len(),
//             value.as_ptr(),
//             value.len() as usize,
//         )
//     };
// }
