use nomos_runtime::child::print;

#[no_mangle]
pub extern "C" fn _run() {
    print("Hello from the child instance!");
}
