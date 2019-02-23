use super::*;

fn parse_template(template: &'static str) -> Template {
    template
        .parse::<CompiledTemplate>()
        .expect("unexpectedly failed to parse template")
        .into()
}

#[test]
fn renders_hello_world() {
    let mut hello = parse_template("Hello, <name>!");
    hello.add("name", "World");
    assert_eq!("Hello, World!", format!("{}", hello.render()));
}

#[test]
fn renders_multiple_attributes() {
    let mut hello = parse_template("Hello, <title><name>!");
    hello.add("name", "World");
    hello.add("title", "Old ");
    assert_eq!("Hello, Old World!", format!("{}", hello.render()));
}

#[test]
fn renders_missing_attributes_as_empty_string() {
    let mut hello = parse_template("Hello, <title><name>!");
    hello.add("name", "World");
    assert_eq!("Hello, World!", format!("{}", hello.render()));
}

fn parse_group(group: &'static str) -> Group {
    match group.parse() {
        Ok(group) => group,
        Err(error) => panic!("unexpectedly failed to parse Group: {}", error),
    }
}

fn get_template(group: &Group, name: &'static str) -> Template {
    group
        .get(name)
        .expect(&format!("unexpectedly failed to get template {}", name))
}

#[test]
fn renders_template_from_group() {
    let group = parse_group(
        r#"
a() ::= "FOO"
"#,
    );
    let a = get_template(&group, "a");
    assert_eq!("FOO", a.render());
}

#[test]
fn renders_template_with_attribute_from_group() {
    let group = parse_group(
        r#"
a(x) ::= "FOO<x>"
"#,
    );
    let mut a = get_template(&group, "a");
    a.add("x", "BAR");
    assert_eq!("FOOBAR", a.render());
}

#[test]
fn renders_template_with_include() {
    let group = parse_group(
        r#"
a() ::= "FOO<b()>"
b() ::= "BAR"
"#,
    );
    let a = get_template(&group, "a");
    assert_eq!("FOOBAR", a.render());
}

#[test]
fn renders_template_with_include_missing_inner_template() {
    let group = parse_group(
        r#"
a() ::= "FOO<b()>"
"#,
    );
    let a = get_template(&group, "a");
    assert_eq!("FOO", a.render());
}
