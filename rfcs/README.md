# String Template RFCs

Many changes, including bug fixes and documentation improvements can
be implemented and reviewed via the normal GitHub pull request
workflow.

Some changes though are "substantial", and we ask that these be put
through a bit of a design process and produce a consensus among the
core team.

## When you need to follow this process

You need to follow this process if you intend to make "substantial"
changes to the String Template language or ecosystem. What constitutes a
"substantial" change is evolving based on community norms and varies
depending on what part of the ecosystem you are proposing to change,
but may include the following.

- Any semantic or syntactic change to the language that is not a
  bugfix.
- Removing language features, including those that are feature-gated.
- Changes to the interfaces used by host languages.

Some changes do not require an RFC:

- Rephrasing, reorganizing, refactoring, or otherwise "changing shape
  does not change meaning".
- Additions that strictly improve objective, numerical quality
  criteria (warning removal, speedup, better platform coverage, more
  parallelism, trap more errors, etc.).
- Additions only likely to be noticed by other
  developers-of-string-template, invisible to
  users-of-string-template.

If you submit a pull request to implement a new feature without going
through the RFC process, it may be closed with a polite request to
submit an RFC first.


## Before creating an RFC

A hastily-proposed RFC can hurt its chances of acceptance. Low quality
proposals, proposals for previously-rejected features, or those that
don't fit into the near-term roadmap, may be quickly rejected, which
can be demotivating for the unprepared contributor. Laying some
groundwork ahead of the RFC can make the process smoother.

Although there is no single way to prepare for submitting an RFC, it
is generally a good idea to pursue feedback from other project
developers beforehand, to ascertain that the RFC may be desirable;
having a consistent impact on the project requires concerted effort
toward consensus-building.

## What the process is

In short, to get a major feature added to String Template, one must
first get the RFC merged into this repository as a markdown file. At
that point the RFC is "active" and may be implemented with the goal of
eventual inclusion into String Template.

# What is this?

This process and set of documents is inspired by the [Rust language
RFC process][rust-rfc] and [Documenting Architecture
Decisions][adr-blog].  It's sort of a fusion of the two processes,
where I've taken the parts I like from each.

[rust-rfc]: https://github.com/rust-lang/rfcs
[adr-blog]: http://thinkrelevance.com/blog/2011/11/15/documenting-architecture-decisions
