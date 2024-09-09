use {
  prettytable::{format, Cell, Row, Table as PrettyTable},
  serde::{de::DeserializeOwned, Deserialize, Serialize},
  std::{
    collections::BTreeMap,
    fmt::{self, Formatter},
  },
  thiserror::Error,
};

#[derive(Error, Debug)]
pub enum DatabaseError {
  #[error("Table '{0}' already exists")]
  TableAlreadyExists(String),
  #[error("Table '{0}' not found")]
  TableNotFound(String),
}

#[derive(Debug, PartialEq)]
struct Table<T: Serialize + DeserializeOwned> {
  name: String,
  rows: Vec<T>,
}

impl<T: Serialize + DeserializeOwned + PrettyPrintable> fmt::Display
  for Table<T>
{
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let mut pretty_table = PrettyTable::new();

    pretty_table.set_format(*format::consts::FORMAT_BOX_CHARS);

    pretty_table.add_row(T::header());

    for row in &self.rows {
      pretty_table.add_row(row.to_row());
    }

    write!(f, "{}", pretty_table)
  }
}

#[derive(Debug, PartialEq)]
struct Database<T: Serialize + DeserializeOwned> {
  tables: BTreeMap<String, Table<T>>,
}

impl<T: Serialize + DeserializeOwned + PrettyPrintable> fmt::Display
  for Database<T>
{
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    let mut output = String::new();

    for (table_name, table) in &self.tables {
      output.push_str(&format!("Table: {}\n", table_name));
      output.push_str(&table.to_string());
      output.push_str("\n\n");
    }

    write!(f, "{}", output.trim_end())
  }
}

impl<T: Serialize + DeserializeOwned> Database<T> {
  fn new() -> Self {
    Self {
      tables: BTreeMap::new(),
    }
  }

  fn create_table(&mut self, name: &str) -> Result<(), DatabaseError> {
    if self.tables.contains_key(name) {
      return Err(DatabaseError::TableAlreadyExists(name.to_string()));
    }

    self.tables.insert(
      name.to_string(),
      Table {
        name: name.to_string(),
        rows: Vec::new(),
      },
    );

    Ok(())
  }

  fn from(&self, table_name: &str) -> Result<&Table<T>, DatabaseError> {
    self
      .tables
      .get(table_name)
      .ok_or_else(|| DatabaseError::TableNotFound(table_name.to_string()))
  }

  fn insert_into(
    &mut self,
    table_name: &str,
    row: T,
  ) -> Result<(), DatabaseError> {
    match self.tables.get_mut(table_name) {
      Some(table) => {
        table.rows.push(row);
        Ok(())
      }
      None => Err(DatabaseError::TableNotFound(table_name.to_string())),
    }
  }

  fn print(&self, table_name: &str) -> Result<(), DatabaseError>
  where
    T: PrettyPrintable,
  {
    print!("{}", self.from(table_name)?);
    Ok(())
  }
}

trait PrettyPrintable {
  fn header() -> Row;
  fn to_row(&self) -> Row;
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Book {
  id: u32,
  name: String,
  author_id: u32,
}

impl PrettyPrintable for Book {
  fn header() -> Row {
    Row::new(vec![
      Cell::new("id"),
      Cell::new("name"),
      Cell::new("author_id"),
    ])
  }

  fn to_row(&self) -> Row {
    Row::new(vec![
      Cell::new(&self.id.to_string()),
      Cell::new(&self.name),
      Cell::new(&self.author_id.to_string()),
    ])
  }
}

fn main() -> Result<(), DatabaseError> {
  let mut database = Database::<Book>::new();

  database.create_table("books")?;

  database.insert_into(
    "books",
    Book {
      id: 1,
      name: "The Elliptical Machine that ate Manhattan".to_string(),
      author_id: 1,
    },
  )?;

  database.insert_into(
    "books",
    Book {
      id: 2,
      name: "Queen of the Bats".to_string(),
      author_id: 2,
    },
  )?;

  database.insert_into(
    "books",
    Book {
      id: 3,
      name: "ChocoMan".to_string(),
      author_id: 3,
    },
  )?;

  database.print("books")?;

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn create_table() {
    let mut db = Database::<Book>::new();
    assert!(db.create_table("books").is_ok());
    assert!(db.create_table("books").is_err());
  }

  #[test]
  fn insert_into() {
    let mut db = Database::<Book>::new();

    db.create_table("books").unwrap();

    assert!(db
      .insert_into(
        "books",
        Book {
          id: 1,
          name: "Test Book".to_string(),
          author_id: 1,
        }
      )
      .is_ok());

    let table = db.from("books").unwrap();

    assert_eq!(table.rows.len(), 1);

    assert_eq!(table.rows[0].name, "Test Book");
  }

  #[test]
  fn insert_into_nonexistent_table() {
    let mut db = Database::<Book>::new();

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
  fn get_table() {
    let mut db = Database::<Book>::new();

    db.create_table("books").unwrap();

    db.insert_into(
      "books",
      Book {
        id: 1,
        name: "Test Book".to_string(),
        author_id: 1,
      },
    )
    .unwrap();

    let table = db.from("books").unwrap();

    assert_eq!(table.name, "books");

    assert_eq!(table.rows.len(), 1);

    assert_eq!(table.rows[0].name, "Test Book");

    assert!(matches!(
      db.from("nonexistent"),
      Err(DatabaseError::TableNotFound(_))
    ));
  }
}
