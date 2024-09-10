[![CI](https://github.com/terror/sql.rs/actions/workflows/ci.yml/badge.svg)](https://github.com/terror/sql.rs/actions/workflows/ci.yml)

This is my attempt at porting [**SQLToy**](https://github.com/weinberg/SQLToy/wiki),
a resource to learn about the implementation for the most common SQL
operators, to Rust 🦀.

The main operators exist as methods implemented on `Database`. Each method takes
table(s) as input and produces a table as its output, making them great for
composition.

Here's an example flow:

```rust
let mut database = Database::new();

let books = database.create_table::<Book>("books").unwrap();

books.borrow_mut().insert_many(&[
  Book {
    id: 1,
    name: "1984".to_string(),
    author_id: 1,
  },
  Book {
    id: 2,
    name: "To Kill a Mockingbird".to_string(),
    author_id: 2,
  },
]);

let authors = database.create_table::<Author>("authors").unwrap();

authors.borrow_mut().insert_many(&[
  Author {
    id: 1,
    name: "George Orwell".to_string(),
  },
  Author {
    id: 2,
    name: "Harper Lee".to_string(),
  },
]);

let result = database.cross_join(&books.borrow(), &authors.borrow());

print!("{result}");
```

Will output something like this:

```
┌────┬───────────────────────┬───────────┬────┬───────────────┐
│ ID │ Name                  │ Author ID │ ID │ Name          │
├────┼───────────────────────┼───────────┼────┼───────────────┤
│ 1  │ 1984                  │ 1         │ 1  │ George Orwell │
├────┼───────────────────────┼───────────┼────┼───────────────┤
│ 1  │ 1984                  │ 1         │ 2  │ Harper Lee    │
├────┼───────────────────────┼───────────┼────┼───────────────┤
│ 2  │ To Kill a Mockingbird │ 2         │ 1  │ George Orwell │
├────┼───────────────────────┼───────────┼────┼───────────────┤
│ 2  │ To Kill a Mockingbird │ 2         │ 2  │ Harper Lee    │
└────┴───────────────────────┴───────────┴────┴───────────────┘
```

## Prior Art

https://github.com/weinberg/SQLToy
