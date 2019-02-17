use parse::st_group;
use string_template::{St, StGroup};

st_group! {
    static ref literal_group {
        a() ::= "foo"
        b() ::= r#"bar "things" { () } () baz => "#
    }
}

fn get_template<'a>(group: &'a StGroup, name: &'static str) -> &'a St {
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
