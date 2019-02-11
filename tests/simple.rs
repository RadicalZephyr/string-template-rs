use string_template::St;

#[test]
fn renders_hello_world() {
    let mut hello = St::new("Hello, <name>!");
    hello.add("name", "World");
    assert_eq!("Hello, World!", format!("{}", hello.render()));
}
