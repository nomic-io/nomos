use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Debug)]
pub enum Action {
    Increment(i32),
    Upgrade(Vec<u8>),
    Execute(Vec<u8>),
}
