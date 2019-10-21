use actions::Action;
use nomos_runtime as runtime;
use runtime::root::{execute, print, read, upgrade, write};

#[no_mangle]
pub extern "C" fn _run() {
    let input_bytes = read(b"input");
    let action: Action = bincode::deserialize(&input_bytes[..]).unwrap();
    run(action);
}

fn run(action: Action) {
    match action {
        Action::Increment(x) => {
            let key = b"count";
            let count: i32 = bincode::deserialize(&read(key)[..]).unwrap();
            let new_count = count + x;
            let encoded_count: Vec<u8> = bincode::serialize(&new_count).unwrap();
            print(&format!("count: {}, new_count: {}, x: {}", count, new_count, x)[..]);
            write(key, encoded_count);
        }
        Action::Upgrade(new_code) => {
            upgrade(new_code);
        }
        Action::Execute(code_vec) => {
            print("got execute action in adder");
            write(b"code_to_execute", code_vec);
            print("wrote code to store");
            execute(b"code_to_execute".to_vec());
            print("called host execute");
        }
    }
}
