# Nomos

_Self-upgrading WebAssembly state machine runtime in Rust_

[![Crate](https://img.shields.io/crates/v/nomos.svg)](https://crates.io/crates/nomos)
[![API](https://docs.rs/nomos/badge.svg)](https://docs.rs/nomos)

**Nomos** is a small library for writing programs that change their own code during execution.

Programs are securely sandboxed inside a WebAssembly virtual machine via [wasmer](https://github.com/wasmerio/wasmer), allowing safe, deterministic, and metered execution of untrusted code. This makes Nomos a great fit for smart contract environments.

Nomos VMs compose recursively. Programs may spawn children with explicit compute and storage limits, and these limits count toward the limits of their parents.

## Usage

**Install:**

```
cargo add nomos
```

**Host:**

```rust
let initial_code = include_bytes!("./path/to/counter_program.wasm");
let vm = nomos::VM::new(initial_code.to_vec());

let initial_count: i32 = 0;
vm.set(b"count", bincode::serialize(&initial_count).unwrap());

let amount_to_add: i32 = 6;
vm.set(b"amount_to_add", bincode::serialize(&amount_to_add).unwrap());

// add 6 twice
vm.call("increment");
vm.call("increment");

let count: i32 = bincode::deserialize(&vm.get(b"count").unwrap());
println!("count is {}", count); // "count is 12"

// Doubler example:
let doubler_code = include_bytes!("./path/to/doubler_program.wasm");
vm.set(b"doubler_program", doubler_code);
vm.set(b"program_to_exec", b"doubler_program");
vm.call("exec_other");
let count: i32 = bincode::deserialize(&vm.get(b"count").unwrap());
println!("count is {}", count); // "count is 24"
```

**Counter WASM program:**

```rust
use nomos_runtime::{read, write, Child}

#[no_mangle]
pub extern "C" fn increment() {
    let count: i32 = bincode::deserialize(&read(b"count").unwrap()).unwrap();
    let inc_amount: i32 = bincode::deserialize(&read(b"amount_to_add").unwrap()).unwrap();
    let result = count + inc_amount;
    write(b"count", bincode::serialize(&result).unwrap());
}

#[no_mangle]
pub extern "C" fn exec_other() {
    let other_program_key = read(b"program_to_exec").unwrap();
    let child = Child::new(other_program_key, b"child_store");
    child.set(b"input", read(b"count").unwrap());
    child.call("run");
    write(b"count", child.get(b"output").unwrap());
}
```

**Doubler WASM program:**

```rust
use nomos_runtime::{read, write}

#[no_mangle]
pub extern "C" fn run() {
    let input: i32 = bincode::deserialize(&read(b"input").unwrap()).unwrap();
    let result = input * 2;
    write(b"output", bincode::serialize(&result).unwrap());
}
```

## Status

Nomos is in early development. Expect bugs and breaking changes.
