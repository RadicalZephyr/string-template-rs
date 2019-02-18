use string_template::{St, StGroup};
use string_template_macro::st_group;

st_group! {
    static ref literal_group {
        a() ::= "foo"
        b() ::= r#"bar "things" { () } () baz => "#
    }
}

fn get_template(group: &StGroup, name: &'static str) -> St {
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
