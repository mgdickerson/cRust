#[derive(Debug, Clone, PartialEq)]
pub struct RetRegister {
    ret: String,
}

impl RetRegister {
    pub fn new() -> Self {
        RetRegister{ ret: String::from("27") }
    }

    pub fn to_string(&self) -> String {
        String::from("R_") + &self.ret
    }
}