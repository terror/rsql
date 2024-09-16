use {
  prettytable::{format, Row as PrettyRow, Table as PrettyTable},
  std::{
    any::Any, cell::RefCell, collections::BTreeMap, fmt, marker::PhantomData,
    rc::Rc,
  },
  thiserror::Error,
};

#[derive(Error, Debug)]
pub enum Error {
  #[error("Table '{0}' already exists")]
  TableAlreadyExists(String),
  #[error("Table '{0}' not found")]
  TableNotFound(String),
  #[error("Invalid row type for table '{0}'")]
  InvalidRowType(String),
}

pub trait Row: fmt::Debug + Clone + Any {
  fn header() -> PrettyRow;
  fn to_pretty_row(&self) -> PrettyRow;
}

#[derive(Debug, Clone)]
pub struct Table<T: Row> {
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

  pub fn insert(&mut self, row: T) {
    self.rows.push(row);
  }

  pub fn insert_many(&mut self, rows: &[T]) {
    self.rows.extend(rows.iter().cloned());
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

#[derive(Debug, Clone)]
pub struct JoinedRow<T: Row, U: Row> {
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
  tables: BTreeMap<String, Rc<RefCell<dyn Any>>>,
}

impl Database {
  pub fn new() -> Self {
    Self {
      tables: BTreeMap::new(),
    }
  }

  pub fn create_table<T: Row + 'static>(
    &mut self,
    name: &str,
  ) -> Result<Rc<RefCell<Table<T>>>, Error> {
    if self.tables.contains_key(name) {
      return Err(Error::TableAlreadyExists(name.to_string()));
    }

    let table = Rc::new(RefCell::new(Table::<T>::new(name.to_string())));

    self.tables.insert(name.to_string(), table.clone());

    Ok(table)
  }

  pub fn cross_join<T: Row + 'static, U: Row + 'static>(
    &self,
    table_a: &Table<T>,
    table_b: &Table<U>,
  ) -> Table<JoinedRow<T, U>> {
    let mut joined_table =
      Table::new(format!("{}_cross_{}", table_a.name, table_b.name));

    for left_row in &table_a.rows {
      for right_row in &table_b.rows {
        joined_table.insert(JoinedRow {
          left: left_row.clone(),
          right: right_row.clone(),
        });
      }
    }

    joined_table
  }

  #[cfg(test)]
  pub fn from<T: Row + 'static>(
    &self,
    table_name: &str,
  ) -> Result<Rc<RefCell<Table<T>>>, Error> {
    match self.tables.get(table_name) {
      Some(table) => table
        .borrow()
        .downcast_ref::<Table<T>>()
        .map(|t| Rc::new(RefCell::new(t.clone())))
        .ok_or_else(|| Error::InvalidRowType(table_name.to_string())),
      None => Err(Error::TableNotFound(table_name.to_string())),
    }
  }
}

#[cfg(test)]
mod tests {
  use {super::*, prettytable::row};

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

  #[test]
  fn create_table() {
    let mut db = Database::new();

    assert!(db.create_table::<Book>("books").is_ok());

    let err = db.create_table::<Book>("books");

    assert!(matches!(err, Err(Error::TableAlreadyExists(_))));
  }

  #[test]
  fn insert_single() {
    let mut db = Database::new();

    let books = db.create_table::<Book>("books").unwrap();

    books.borrow_mut().insert(Book {
      id: 1,
      name: "Test Book".to_string(),
      author_id: 1,
    });

    let table = db.from::<Book>("books").unwrap();

    assert_eq!(table.borrow().rows.len(), 1);

    assert_eq!(table.borrow().rows[0].name, "Test Book");
  }

  #[test]
  fn insert_many() {
    let mut db = Database::new();

    let books = db.create_table::<Book>("books").unwrap();

    books.borrow_mut().insert_many(&[
      Book {
        id: 1,
        name: "Book 1".to_string(),
        author_id: 1,
      },
      Book {
        id: 2,
        name: "Book 2".to_string(),
        author_id: 2,
      },
    ]);

    let table = db.from::<Book>("books").unwrap();

    assert_eq!(table.borrow().rows.len(), 2);

    assert_eq!(table.borrow().rows[0].name, "Book 1");

    assert_eq!(table.borrow().rows[1].name, "Book 2");
  }

  #[test]
  fn cross_join() {
    let mut db = Database::new();

    let books = db.create_table::<Book>("books").unwrap();

    books.borrow_mut().insert_many(&[
      Book {
        id: 1,
        name: "1984".to_string(),
        author_id: 1,
      },
      Book {
        id: 2,
        name: "Animal Farm".to_string(),
        author_id: 1,
      },
    ]);

    let authors = db.create_table::<Author>("authors").unwrap();

    authors.borrow_mut().insert_many(&[
      Author {
        id: 1,
        name: "George Orwell".to_string(),
      },
      Author {
        id: 2,
        name: "Aldous Huxley".to_string(),
      },
    ]);

    let joined_table = db.cross_join(&books.borrow(), &authors.borrow());

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
  }
}
