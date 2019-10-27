use nomos_runtime::{print, read, write};

#[no_mangle]
pub extern "C" fn _double() {
  let input_bytes = read(b"input").unwrap();
  let x: i32 = bincode::deserialize(&input_bytes).unwrap();
  let result = x * 2;
  let result_bytes = bincode::serialize(&result).unwrap();
  write(b"output".to_vec(), result_bytes);
}
