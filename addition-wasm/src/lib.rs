use actions::Action;
use nomos_runtime::{execute, print, read, upgrade, write, Child};

#[no_mangle]
pub extern "C" fn _run() {
    let input_bytes = read(b"input").unwrap();
    let action: Action = bincode::deserialize(&input_bytes[..]).unwrap();
    run(action);
}

fn run(action: Action) {
    match action {
        Action::Increment(x) => {
            let key = b"count";
            let count: i32 = bincode::deserialize(&read(key).unwrap()[..]).unwrap();
            let new_count = count + x;
            let encoded_count: Vec<u8> = bincode::serialize(&new_count).unwrap();
            print(&format!("count: {}, new_count: {}, x: {}", count, new_count, x)[..]);
            write(key, encoded_count);
        }
        Action::Upgrade(new_code) => {
            upgrade(new_code);
        }
        Action::Execute(code_vec) => {
            write(b"code_to_execute", code_vec);
            let x = 4;
            let x_bytes = bincode::serialize(&x).unwrap();
            let child = Child::new(b"code_to_execute".to_vec(), b"child_store".to_vec());
            child.set(b"x".to_vec(), x_bytes);
            child.call("_run");
            let result = child.get(b"x".to_vec()).unwrap();
            print(&format!("Value of child.foo after call: {:?}", result)[..]);
        }
    }
}
