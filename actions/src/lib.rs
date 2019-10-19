#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Debug)]
pub enum Action {
    Increment(i32),
    Upgrade(Vec<u8>),
}
