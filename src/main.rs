use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;

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

#[derive(Debug, PartialEq)]
struct Database<T: Serialize + DeserializeOwned> {
  tables: BTreeMap<String, Table<T>>,
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

  #[cfg(test)]
  fn get_table(&self, table_name: &str) -> Result<&Table<T>, DatabaseError> {
    self
      .tables
      .get(table_name)
      .ok_or_else(|| DatabaseError::TableNotFound(table_name.to_string()))
  }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Person {
  name: String,
  age: u8,
  department_id: u8,
}

fn main() -> Result<(), DatabaseError> {
  let mut database = Database::<Person>::new();

  database.create_table("people")?;

  database.insert_into(
    "people",
    Person {
      name: "Alice Smith".to_string(),
      age: 30,
      department_id: 1,
    },
  )?;

  database.insert_into(
    "people",
    Person {
      name: "Bob Johnson".to_string(),
      age: 25,
      department_id: 2,
    },
  )?;

  println!("{:#?}", database);

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn create_table() {
    let mut db = Database::<Person>::new();
    assert!(db.create_table("employees").is_ok());
    assert!(db.create_table("employees").is_err());
  }

  #[test]
  fn insert_into() {
    let mut db = Database::<Person>::new();

    db.create_table("employees").unwrap();

    assert!(db
      .insert_into(
        "employees",
        Person {
          name: "John Doe".to_string(),
          age: 30,
          department_id: 1,
        }
      )
      .is_ok());

    let table = db.get_table("employees").unwrap();

    assert_eq!(table.rows.len(), 1);

    assert_eq!(table.rows[0].name, "John Doe");
  }

  #[test]
  fn insert_into_nonexistent_table() {
    let mut db = Database::<Person>::new();

    assert!(matches!(
      db.insert_into(
        "nonexistent",
        Person {
          name: "Jane Doe".to_string(),
          age: 25,
          department_id: 2,
        }
      ),
      Err(DatabaseError::TableNotFound(_))
    ));
  }

  #[test]
  fn get_table() {
    let mut db = Database::<Person>::new();

    db.create_table("employees").unwrap();

    db.insert_into(
      "employees",
      Person {
        name: "Alice".to_string(),
        age: 28,
        department_id: 3,
      },
    )
    .unwrap();

    let table = db.get_table("employees").unwrap();

    assert_eq!(table.name, "employees");

    assert_eq!(table.rows.len(), 1);

    assert_eq!(table.rows[0].name, "Alice");

    assert!(matches!(
      db.get_table("nonexistent"),
      Err(DatabaseError::TableNotFound(_))
    ));
  }
}
