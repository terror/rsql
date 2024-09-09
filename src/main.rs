use {
  prettytable::{format, row, Row as PrettyRow, Table as PrettyTable},
  std::{any::Any, collections::BTreeMap, fmt, marker::PhantomData},
  thiserror::Error,
};

trait Row: fmt::Debug + Clone + Any {
  fn header() -> PrettyRow;
  fn to_pretty_row(&self) -> PrettyRow;
}

struct Table<T: Row> {
  name: String,
  rows: Vec<T>,
  phantom: PhantomData<fn() -> T>,
}

impl<T: Row> Table<T> {
  fn new(name: String) -> Self {
    Self {
      name,
      rows: Vec::new(),
      phantom: PhantomData,
    }
  }
}

impl<T: Row> fmt::Display for Table<T> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let mut pretty_table = PrettyTable::new();

    pretty_table.set_format(*format::consts::FORMAT_BOX_CHARS);
    pretty_table.set_titles(T::header());

    for row in &self.rows {
      pretty_table.add_row(row.to_pretty_row());
    }

    write!(f, "{}", pretty_table)
  }
}

#[derive(Error, Debug)]
enum DatabaseError {
  #[error("Table '{0}' already exists")]
  TableAlreadyExists(String),
  #[error("Table '{0}' not found")]
  TableNotFound(String),
  #[error("Invalid row type for table '{0}'")]
  InvalidRowType(String),
}

#[derive(Debug, Clone)]
struct JoinedRow<T: Row, U: Row> {
  left: T,
  right: U,
}

impl<T: Row, U: Row> Row for JoinedRow<T, U> {
  fn header() -> PrettyRow {
    let mut header = T::header();
    header.extend(U::header().iter().cloned());
    header
  }

  fn to_pretty_row(&self) -> PrettyRow {
    let mut row = self.left.to_pretty_row();
    row.extend(self.right.to_pretty_row().iter().cloned());
    row
  }
}

#[derive(Default)]
pub struct Database {
  tables: BTreeMap<String, Box<dyn Any>>,
}

impl Database {
  fn new() -> Self {
    Self {
      tables: BTreeMap::new(),
    }
  }

  fn create_table<T: Row + 'static>(
    &mut self,
    name: &str,
  ) -> Result<(), DatabaseError> {
    if self.tables.contains_key(name) {
      return Err(DatabaseError::TableAlreadyExists(name.to_string()));
    }

    let table = Table::<T>::new(name.to_string());

    self.tables.insert(name.to_string(), Box::new(table));

    Ok(())
  }

  fn cross_join<T: Row + 'static, U: Row + 'static>(
    &self,
    table_a: &str,
    table_b: &str,
  ) -> Result<Table<JoinedRow<T, U>>, DatabaseError> {
    let table_a = self.from::<T>(table_a)?;
    let table_b = self.from::<U>(table_b)?;

    let mut joined_table =
      Table::new(format!("{}_cross_{}", table_a.name, table_b.name));

    for left_row in table_a.rows.clone() {
      for right_row in table_b.rows.clone() {
        joined_table.rows.push(JoinedRow {
          left: left_row.clone(),
          right: right_row.clone(),
        });
      }
    }

    Ok(joined_table)
  }

  fn from<T: Row + 'static>(
    &self,
    table_name: &str,
  ) -> Result<&Table<T>, DatabaseError> {
    match self.tables.get(table_name) {
      Some(table) => table
        .downcast_ref::<Table<T>>()
        .ok_or_else(|| DatabaseError::InvalidRowType(table_name.to_string())),
      None => Err(DatabaseError::TableNotFound(table_name.to_string())),
    }
  }

  fn insert_into<T: Row + 'static>(
    &mut self,
    table_name: &str,
    row: T,
  ) -> Result<(), DatabaseError> {
    match self.tables.get_mut(table_name) {
      Some(table) => {
        let table = table.downcast_mut::<Table<T>>().ok_or_else(|| {
          DatabaseError::InvalidRowType(table_name.to_string())
        })?;

        table.rows.push(row);

        Ok(())
      }
      None => Err(DatabaseError::TableNotFound(table_name.to_string())),
    }
  }

  fn print<T: Row + 'static>(
    &self,
    table_name: &str,
  ) -> Result<(), DatabaseError> {
    match self.tables.get(table_name) {
      Some(table) => {
        let table = table.downcast_ref::<Table<T>>().ok_or_else(|| {
          DatabaseError::InvalidRowType(table_name.to_string())
        })?;

        print!("{table_name}\n{table}");

        Ok(())
      }
      None => Err(DatabaseError::TableNotFound(table_name.to_string())),
    }
  }
}

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

fn main() -> Result<(), DatabaseError> {
  let mut database = Database::new();

  database.create_table::<Book>("books")?;
  database.create_table::<Author>("authors")?;

  database.insert_into(
    "books",
    Book {
      id: 1,
      name: "1984".to_string(),
      author_id: 1,
    },
  )?;

  database.insert_into(
    "books",
    Book {
      id: 2,
      name: "To Kill a Mockingbird".to_string(),
      author_id: 2,
    },
  )?;

  database.insert_into(
    "authors",
    Author {
      id: 1,
      name: "George Orwell".to_string(),
    },
  )?;

  database.insert_into(
    "authors",
    Author {
      id: 2,
      name: "Harper Lee".to_string(),
    },
  )?;

  database.print::<Book>("books")?;
  database.print::<Author>("authors")?;

  print!(
    "Cross Joined (books, authors):\n{}",
    database.cross_join::<Book, Author>("books", "authors")?
  );

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn create_table() {
    let mut db = Database::new();

    assert!(db.create_table::<Book>("books").is_ok());

    let err = db.create_table::<Book>("books");

    assert!(matches!(err, Err(DatabaseError::TableAlreadyExists(_))));
  }

  #[test]
  fn insert_into() {
    let mut db = Database::new();

    db.create_table::<Book>("books").unwrap();

    assert!(db
      .insert_into(
        "books",
        Book {
          id: 1,
          name: "Test Book".to_string(),
          author_id: 1
        }
      )
      .is_ok());

    let table = db.from::<Book>("books").unwrap();

    assert_eq!(table.rows.len(), 1);
    assert_eq!(table.rows[0].name, "Test Book");
  }

  #[test]
  fn insert_into_nonexistent_table() {
    let mut db = Database::new();

    let err = db.insert_into(
      "nonexistent",
      Book {
        id: 1,
        name: "Test Book".to_string(),
        author_id: 1,
      },
    );

    assert!(matches!(err, Err(DatabaseError::TableNotFound(_))));
  }

  #[test]
  fn insert_wrong_type() {
    let mut db = Database::new();

    db.create_table::<Book>("books").unwrap();

    let err = db.insert_into(
      "books",
      Author {
        id: 1,
        name: "Test Author".to_string(),
      },
    );

    assert!(matches!(err, Err(DatabaseError::InvalidRowType(_))));
  }

  #[test]
  fn get_table() {
    let mut db = Database::new();

    db.create_table::<Book>("books").unwrap();

    db.insert_into(
      "books",
      Book {
        id: 1,
        name: "Test Book".to_string(),
        author_id: 1,
      },
    )
    .unwrap();

    let table = db.from::<Book>("books").unwrap();

    assert_eq!(table.name, "books");
    assert_eq!(table.rows.len(), 1);
    assert_eq!(table.rows[0].name, "Test Book");

    assert!(matches!(
      db.from::<Book>("nonexistent"),
      Err(DatabaseError::TableNotFound(_))
    ));
  }

  #[test]
  fn get_table_wrong_type() {
    let mut db = Database::new();

    db.create_table::<Book>("books").unwrap();

    assert!(matches!(
      db.from::<Author>("books"),
      Err(DatabaseError::InvalidRowType(_))
    ));
  }

  #[test]
  fn cross_join() {
    let mut db = Database::new();

    db.create_table::<Book>("books").unwrap();
    db.create_table::<Author>("authors").unwrap();

    db.insert_into(
      "books",
      Book {
        id: 1,
        name: "1984".to_string(),
        author_id: 1,
      },
    )
    .unwrap();

    db.insert_into(
      "books",
      Book {
        id: 2,
        name: "Animal Farm".to_string(),
        author_id: 1,
      },
    )
    .unwrap();

    db.insert_into(
      "authors",
      Author {
        id: 1,
        name: "George Orwell".to_string(),
      },
    )
    .unwrap();

    db.insert_into(
      "authors",
      Author {
        id: 2,
        name: "Aldous Huxley".to_string(),
      },
    )
    .unwrap();

    let joined_table =
      db.cross_join::<Book, Author>("books", "authors").unwrap();

    assert_eq!(joined_table.rows.len(), 4);

    let first_row = &joined_table.rows[0];

    assert_eq!(first_row.left.id, 1);
    assert_eq!(first_row.left.name, "1984");
    assert_eq!(first_row.left.author_id, 1);
    assert_eq!(first_row.right.id, 1);
    assert_eq!(first_row.right.name, "George Orwell");

    let last_row = &joined_table.rows[3];

    assert_eq!(last_row.left.id, 2);
    assert_eq!(last_row.left.name, "Animal Farm");
    assert_eq!(last_row.left.author_id, 1);
    assert_eq!(last_row.right.id, 2);
    assert_eq!(last_row.right.name, "Aldous Huxley");

    let combinations = [
      ("1984", "George Orwell"),
      ("1984", "Aldous Huxley"),
      ("Animal Farm", "George Orwell"),
      ("Animal Farm", "Aldous Huxley"),
    ];

    for (book, author) in combinations.iter() {
      assert!(joined_table
        .rows
        .iter()
        .any(|row| row.left.name == *book && row.right.name == *author));
    }
  }
}
