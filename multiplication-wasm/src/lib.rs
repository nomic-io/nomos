use nomos_runtime::{print, read, write};

#[no_mangle]
pub extern "C" fn _run() {
  print("hello from the child");
  let x_bytes = read(b"x".to_vec()).unwrap();
  print("got x bytes");
  let x: i32 = bincode::deserialize(&x_bytes).unwrap();
  print("deserialized x");
  let result = x * 2;
  let result_bytes = bincode::serialize(&result).unwrap();
  write(b"x".to_vec(), result_bytes);
  // let input_bytes = read_input();
  // let x: i32 = bincode::deserialize(&input_bytes[..]).unwrap();
  // let result = x * 2;
  // let result_bytes = bincode::serialize(&result).unwrap();
}
