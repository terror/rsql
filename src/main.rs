use {
  prettytable::{row, Row as PrettyRow},
  rsql::{Database, Row},
};

#[derive(Debug, Clone, PartialEq)]
struct Book {
  id: u32,
  name: String,
  author_id: u32,
}

impl Row for Book {
  fn header() -> PrettyRow {
    row!["ID", "Name", "Author ID"]
  }

  fn to_pretty_row(&self) -> PrettyRow {
    row![self.id, self.name, self.author_id]
  }
}

#[derive(Debug, Clone, PartialEq)]
struct Author {
  id: u32,
  name: String,
}

impl Row for Author {
  fn header() -> PrettyRow {
    row!["ID", "Name"]
  }

  fn to_pretty_row(&self) -> PrettyRow {
    row![self.id, self.name]
  }
}

fn main() {
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

  print!(
    "{}",
    database.cross_join(&books.borrow(), &authors.borrow())
  );
}
