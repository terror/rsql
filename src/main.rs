use {
  prettytable::{format, row, Row, Table as PrettyTable},
  std::{any::Any, collections::BTreeMap, fmt, marker::PhantomData},
  thiserror::Error,
};

pub trait TableRow: fmt::Debug + Any {
  fn as_any(&self) -> &dyn Any;
  fn header() -> Row;
  fn to_pretty_row(&self) -> Row;
}

struct Table<T: TableRow> {
  #[allow(dead_code)]
  name: String,
  rows: Vec<T>,
  phantom: PhantomData<fn() -> T>,
}

impl<T: TableRow> Table<T> {
  fn new(name: String) -> Self {
    Self {
      name,
      rows: Vec::new(),
      phantom: PhantomData,
    }
  }
}

impl<T: TableRow> fmt::Display for Table<T> {
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
pub enum DatabaseError {
  #[error("Table '{0}' already exists")]
  TableAlreadyExists(String),
  #[error("Table '{0}' not found")]
  TableNotFound(String),
  #[error("Invalid row type for table '{0}'")]
  InvalidRowType(String),
}

#[derive(Default)]
pub struct Database {
  tables: BTreeMap<String, Box<dyn Any>>,
}

impl Database {
  pub fn new() -> Self {
    Self {
      tables: BTreeMap::new(),
    }
  }

  fn create_table<T: TableRow + 'static>(
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

  #[cfg(test)]
  fn from<T: TableRow + 'static>(
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

  fn insert_into<T: TableRow + 'static>(
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

  fn print<T: TableRow + 'static>(
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

#[derive(Debug, PartialEq)]
struct Book {
  id: u32,
  name: String,
  author_id: u32,
}

impl TableRow for Book {
  fn as_any(&self) -> &dyn Any {
    self
  }

  fn header() -> Row {
    row!["ID", "Name", "Author ID"]
  }

  fn to_pretty_row(&self) -> Row {
    row![self.id, self.name, self.author_id]
  }
}

#[derive(Debug, PartialEq)]
struct Author {
  id: u32,
  name: String,
}

impl TableRow for Author {
  fn as_any(&self) -> &dyn Any {
    self
  }

  fn header() -> Row {
    row!["ID", "Name"]
  }

  fn to_pretty_row(&self) -> Row {
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
}
