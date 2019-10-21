mod nomos;

#[cfg(test)]
mod tests {
    use crate::nomos;
    #[test]
    fn my_test() {
        let addition_bytes =
            include_bytes!("../../target/wasm32-unknown-unknown/release/addition_wasm.wasm");
        let multiply_bytes =
            include_bytes!("../../target/wasm32-unknown-unknown/release/multiplication_wasm.wasm");
        let mut vm = nomos::VM::new(addition_bytes.to_vec());
        // let msg = nomos::Action::Increment(12);
        let increment_msg = actions::Action::Increment(6);

        let increment_msg_bytes = bincode::serialize(&increment_msg).unwrap();

        vm.next(&increment_msg_bytes);
        vm.next(&increment_msg_bytes);

        // let decoded_msg: actions::Action = bincode::deserialize(&msg_bytes[..]).unwrap();

        let execute_action = actions::Action::Execute(multiply_bytes.to_vec());
        let execute_action_bytes = bincode::serialize(&execute_action).unwrap();

        vm.next(&execute_action_bytes);

        println!(
            "count, after adding 6 three times: {:?}",
            vm.state.get(&b"count".to_vec())
        );
    }
}
