mod nomos;

#[cfg(test)]
mod tests {
    use crate::nomos;
    #[test]
    fn my_test() {
        let addition_bytes = include_bytes!(
            "../../addition-wasm/target/wasm32-unknown-unknown/release/addition_wasm.wasm"
        );
        let multiply_bytes = include_bytes!(
            "../../multiplication-wasm/target/wasm32-unknown-unknown/release/multiplication_wasm.wasm"
        );
        let mut vm = nomos::VM::new(addition_bytes.to_vec());
        // let msg = nomos::Action::Increment(12);
        let increment_msg = actions::Action::Increment(6);

        let increment_msg_bytes = bincode::serialize(&increment_msg).unwrap();

        vm.set(b"input", increment_msg_bytes);
        let initial_count: i32 = 0;
        vm.set(b"count", bincode::serialize(&initial_count).unwrap());
        vm.call("_run");
        vm.call("_run");

        let execute_action = actions::Action::Execute(multiply_bytes.to_vec());
        let execute_action_bytes = bincode::serialize(&execute_action).unwrap();
        vm.set(b"input", execute_action_bytes);
        vm.call("_run");
        let count_bytes = vm.get(b"count").unwrap();
        let count: i32 = bincode::deserialize(&count_bytes).unwrap();
        println!("count after adding 6 twice then doubling: {}", count);
    }
}
