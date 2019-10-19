mod nomos;

#[cfg(test)]
mod tests {
    use crate::nomos;
    #[test]
    fn my_test() {
        let addition_bytes = include_bytes!(
            "../../addition-wasm/target/wasm32-unknown-unknown/release/addition_wasm.wasm"
        );
        let mut vm = nomos::VM::new(addition_bytes.to_vec());
        // let msg = nomos::Action::Increment(12);
        let increment_msg = actions::Action::Increment(6);

        let increment_msg_bytes = bincode::serialize(&increment_msg).unwrap();

        vm.next(&increment_msg_bytes);
        vm.next(&increment_msg_bytes);

        // let decoded_msg: actions::Action = bincode::deserialize(&msg_bytes[..]).unwrap();

        // TODO: point to multiplication wasm program.
        let multiplication_bytes = include_bytes!(
            "../../multiplication-wasm/target/wasm32-unknown-unknown/release/multiplication_wasm.wasm"
        );
        let upgrade_msg = actions::Action::Upgrade(multiplication_bytes.to_vec());
        let upgrade_msg_bytes = bincode::serialize(&upgrade_msg).unwrap();

        let upgrade_action: actions::Action = bincode::deserialize(&upgrade_msg_bytes[..]).unwrap();

        vm.next(&upgrade_msg_bytes);
        vm.next(&increment_msg_bytes);

        println!(
            "count, after adding 6 twice, then multiplying by 6: {:?}",
            vm.state.get(&b"count".to_vec())
        );
    }
}
