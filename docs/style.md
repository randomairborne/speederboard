# Basic speederboard style guide

## Formatting

This project mostly adheres to rustfmt. `cargo +nightly fmt` is the canonical format command.

This will not affect SQL strings. SQL queries should be broken up with a new UPPERCASE
KEYWORD on the new lines, with the new lines indented the same amount as rustfmt indents
the start of the string. On joined tables, newlines should be inserted between selects of
multiple tables.

## Naming

templating and form structs should be named with this pattern:
`(sane endpoint name)(Form|Query|Page)` and **must be public**
