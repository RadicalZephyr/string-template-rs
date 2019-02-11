pub struct St {}

impl St {
    pub fn new(_template: impl Into<String>) -> St {
        St {}
    }

    pub fn add(&mut self, _name: impl Into<String>, _value: impl Into<String>) {}

    pub fn render(&self) -> String {
        String::new()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
