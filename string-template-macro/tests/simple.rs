use string_template::{Group, Template};
use string_template_macro::st_group;
use string_template_test::TemplateTestExt as _;

st_group! {
    static ref literal_group {
        a() ::= "foo"
        b() ::= r#"bar "things" { () } () baz => "#
        c(x) ::= "<x>"
    }
}

fn get_template(group: &Group, name: &'static str) -> Template {
    group
        .get(name)
        .expect(&format!("unexpectedly failed to find template {}", name))
}

#[test]
fn parse_literal_group() {
    let a = get_template(&literal_group, "a");
    assert_eq!("foo", format!("{}", a.render()));

    let b = get_template(&literal_group, "b");
    assert_eq!(
        r#"bar "things" { () } () baz => "#,
        format!("{}", b.render())
    );
}

#[test]
fn use_static_methods() {
    let a = literal_group.a();
    assert_eq!("foo", format!("{}", a.render()));

    let b = literal_group.b();
    assert_eq!(
        r#"bar "things" { () } () baz => "#,
        format!("{}", b.render())
    );
}

#[test]
fn render_attributes() {
    let mut c = literal_group.c();
    c.add_expect("x", "Things");
    assert_eq!("Things", c.render());
}
