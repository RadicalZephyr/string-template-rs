- Title: free_spaced_syntax
- Date: 2019-02-24
- Status: Proposed

# Summary
[summary]: #summary

Introduce an expanded form of the ST language syntax that is easier to
read and more clearly differentiates the difference between literal
strings and template expressions.

# Motivation
[motivation]: #motivation

Much like regular expressions, the ST language is concise and very
information-dense.  This is great for simple uses, but more complex
examples can become very difficult to read.

Many regular expression libraries allow for creating ["free-spacing"
regular expressions][freespace-regex] because they are _much easier to
read_. This new syntax is intended to do the same for creating ST
templates by:

- Visually differentiating literal text content to be rendered from
  template expressions.
- Allow the user to include arbitrary non-rendering whitespace within
  template definitions.
- Allow the user to include non-rendering comments within template
  definitions.
- Remove the need for parsing against arbitrary template escape
  strings.

This expanded syntax is part of a larger re-imagining of ST as more of
a language and less of a template library.

[freespace-regex]: https://www.regular-expressions.info/freespacing.html

# Guide-level explanation
[guide-level-explanation]: #guide-level-explanation

This introduces an entirely new syntax for specifying a template
body. By analogy, we call this a free-spaced body. This new syntax
flips the priority of the other two template body syntaxes `<<...>>`
and `"..."` where all content is assumed to be literal unless escaped
by the set template delimiters (which default to `<...>`).

The new syntax assumes that all content is a template expression
unless it is surrounded by double-quote characters (`"..."`). In
addition, new syntactic forms are introduced for many template
expressions like template application/mapping.

## Basics - Literals, Whitespace, Comments

A free-spaced template body is always surrounded by braces
(`{...}`). Inside the braces whitespace is ignored unless it occurs
inside a pair of double-quotes (`"..."`). So the classic "Hello World"
template looks like this:

```
hello() ::= {
    "Hello World!"
}
```

Adjacent template expressions of any type are concatenated during
rendering. Any amount of separating whitespace between two template
expressions is completely elided, just as the template expression
delimiters `<` and `>` do not appear in short form ST templates. In
addition, comments are allowed inside the block. Comments start with a
pair of forward-slashes `//` and continue until the end of the line.

Knowing all this, we can see that the following example produces
identical output to the previous example since no new whitespace
appears inside of a string literal.

```
hello() ::= {
    "Hello"  // These are comments.
    " "      // This is the only whitespace in the output
    "World!"
}
```

In addition to ignored whitespace and literal content, a free-spaced
body can directly contain many of the normal ST expressions that would
normally need to be escaped with the `<..>` delimiters. So
interpolating a template include, attribute, or attribute field access
is as simple as writing it. So rendering the following template with
the attributes set as indicated would render `"Hello World!"`.

```
greetings = {"hello": "Hello"}
place = "World"
-----

hello(greetings, place) ::= {
    greetings.hello " " place bang()
}
bang() ::= "!"
```

## Mixing Body Formats

The new free-spaced body syntax is applied on a per-template basis,
and different templates within a group can utilize any of the
supported body syntaxes regardless of what other templates in the
group use.

```
a() ::= { "free-spaced bodies" b() }
b() ::= "are not the only <c()>"
c() ::= <<syntax that can be used>>
```

## Conditionals

Conditionals have a slightly cleaner syntax in free-spaced form. The
style is based on Rust's syntax for conditionals. Parentheses
surrounding the boolean expression are optional, the braces
surrounding the body of each branch are not.

```
a() ::= {
    if x && y { "x & y content" }
    else if x { "x content" }
    else if y { "y content" }
    else { "else content" }
}
```

## Anonymous/lambda templates

The syntax for anonymous templates changes significantly in
free-spaced syntax. The new syntax is heavily inspired by Rust's
lambda syntax. Arguments of the template are found between a pair of
vertical bars `|...|` and the body of the template can be either an
un-delimited single expression (literal, attribute reference, mapping
etc.) or a brace delimited body in free-spaced syntax.

```
{ x | x }  // dense syntax form
|x| x // minimal identity template
|x| { x }  // identity template with braces
|| { "foo" } // literal template with no formal arguments
|x, y| { x " " y }  // multi-expression body
```


## Mapping/template application

The syntax for mapping in ST is very dense and uses an absolute
minimum of characters to convey a wide variety of possible
invocations. The free-spaced syntax for mapping is much more verbose,
but should be much more readable even to someone unfamiliar with the
ST language.  The syntax is the keyword `map` followed by an ordered
comma-separated list of attribute names or attribute field
accesses. The list of attribute names is followed by the keyword
`with` and then an expression to map across each value. The simplest
expression to map is a named template.

```
a(xs) ::= {
    map xs with parens()
}
parens(x) ::= "(<x>)"
```

Applying template "a" to `["a", "b", "c"]` would yield `(a)(b)(c)`.

Multiple templates can be applied to each element of the mapped
attribute by using a bracket delimited list of template names instead.

```
a(xs) ::= {
    map xs with [ id(), parens() ]
}
id(x) ::= { x }
parens(x) ::= "(<x>)"
```

Applying template "a" to `["a", "b", "c"]` would yield `a(a)b(b)c(c)`.

The mapping construct can also walk multiple multi-valued attributes in
lock-step by providing a comma separated list of attributes to parse
between the keywords `map` and `with`.

```
a(xs, ys) ::= { map xs, ys with foo() }
foo(x, y) ::= "<x><y>"
```

Anonymous templates may be used in the same position as template
names:

```
a(xs) ::= { map xs with |x| { "(" x ")" } }
a(xs) ::= {
  map xs with [
    parens(),
    |x| { "[" x "]" }
  ]
 }
parens(x) ::= "(<x>)"
```

An optional separator can be supplied for a mapping operation by
inserting the compound keyword `join with` followed by a literal
string between the last attribute name and the opening bracket.

```
a(xs) ::= {
    map xs join with ", " with parens()
}
parens(x) ::= "(<x>)"
```

Applying template "a" to `["a", "b", "c"]` would yield `(a), (b),
(c)`.

The same `join with` suffix can be applied to join multi-valued
attributes with a specified separator.

```
a(xs) ::= { xs join with ", " }
```

Applying template "a" to `["a", "b", "c"]` would yield `a,b,c`.


# Reference-level explanation
[reference-level-explanation]: #reference-level-explanation

The new free-spaced syntax is an alternative syntax for specifying
template bodies, and as such has no interaction with the syntaxes of
the other two languages. No new functionality is provided beyond what
can already be accomplished in previously available ST syntax, and no
operations should be missing.

# Drawbacks
[drawbacks]: #drawbacks

The main drawback is that it is an entirely new set of syntax to
parse.

It's possible that this new syntax will not see much use, or possibly
worse, that it will enjoy too MUCH usage.  Because it is possible to
make complex templates more readable with free-spaced syntax, there is
the non-trivial possibility that more complex templates will be
created because this syntax makes it possible to continue to read and
modify them. Ultimately I view this as more of a feature than a bug.

# Rationale and alternatives
[rationale-and-alternatives]: #rationale-and-alternatives

Several alternative syntaxes where explored for mapping. Omitting the
template name and argument definition:

All the following examples are alternative syntax proposals for the
equivalent of the first template in the old style ST syntax.

```
"<foo:{f | <f>}>" // original syntax for applying an anonymous template

{ foo : |f| f } // initial direct translation from standard syntax
{ for f in foo { f } } // for style syntax
{ map |f| { f } over foo } // first map keyword style
{ map onto foo |f| { f } }
{ map foo onto |f| { f } }
{ map foo |f| { f } }
{ map foo with |f| { f } }
```

And for mapping with a separator:

```
"<foo:bar();separator=\",\">"
{ map foo with separator="," { bar() } }
{ map(separator=",") foo { bar() } }
{ map(",") foo { bar() } }
{ map by "," foo { bar() } }
```

The keyword `map` was chosen over `for` to avoid the procedural coding
associations of a for-loop.  Since the goal of providing this syntax
was primarily improved readability the syntax that read most naturally
was chosen.


# Prior art
[prior-art]: #prior-art

As stated, this new syntax is somewhat of an analog to the free-spaced
syntax introduced for regular expressions. regular-expressions.info
claims that "most modern regex flavors" support free-spacing mode. In
my personal experience as a developer, I have only written free-spaced
regexes a bare handful of times, and I have never encountered them in
the wild. There are many possible reasons for this, including lack of
awareness about the feature, lack of availability in particular regex
engines, or lack of need.

In addition, experience shows that most templates consist of large
amounts of literal text with small amounts of template expressions
scattered throughout. For this reason the original syntaxes are not
being deprecated or removed. The addition of free-spaced syntax is
largely to accommodate the minor use-case of conditional or mapping
heavy templates in a form that is more easily readable.

I believe it will be best practice to not have much literal content in
free-spaced template bodies, and instead put most literal content into
classic ST template bodies and then use these via template include
inside the free-spaced template.


# Unresolved questions
[unresolved-questions]: #unresolved-questions

Since the parser for this syntax has not yet been written there may be
changes required to the syntax as described above to facilitate the
implementation. However, deviations should be fairly minor.

Also, not all syntactic elements have been reviewed for readability in
the free-spaced syntax. Before this is implemented these will have to
be addressed.


# Future possibilities
[future-possibilities]: #future-possibilities

It would be possible to extend the flexibility of mapping by allowing
a Python list-comprehension like syntax for the attribute expressions.

So instead of:

```
a(xs, ys) ::= { map xs, ys with foo() }
foo(x, y) ::= "(<x>,<y>)"
```

Where elements from `xs` will always become the `x` argument in `foo`,
one could write this:

```
a(xs, ys) ::= { map x in xs, y in ys with foo(y, x) }
foo(y, x) ::= "(<x>,<y>)"
```

This is not strictly necessary and might be confusing.  This kind of
behavior could be approximated simply by creating an anonymous
template:

```
a(xs, ys) ::= { map xs, ys with |x,y| foo(y,x) }
foo(y, x) ::= "(<x>,<y>)"
```
