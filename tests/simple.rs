use string_template::St;

#[test]
fn renders_hello_world() {
    let mut hello = St::new("Hello, <name>!");
    hello.add("name", "World");
    assert_eq!("Hello, World!", format!("{}", hello.render()));
}

#[test]
fn renders_multiple_attributes() {
    let mut hello = St::new("Hello, <title><name>!");
    hello.add("name", "World");
    hello.add("title", "Old ");
    assert_eq!("Hello, Old World!", format!("{}", hello.render()));
}

#[test]
fn renders_missing_attributes_as_empty_string() {
    let mut hello = St::new("Hello, <title><name>!");
    hello.add("name", "World");
    assert_eq!("Hello, World!", format!("{}", hello.render()));
}
