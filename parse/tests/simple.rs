use parse::st_group;

st_group! {
    static ref simple_group {
         a() ::= <<foo>>;
    }
}

#[test]
fn parse_literal_group() {
    let a = simple_group
        .get("a")
        .expect("unexpectedly failed to find template a");
    assert_eq!("foo", format!("{}", a.render()));
}
