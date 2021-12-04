use std::error::Error;
use std::fmt;

#[derive(Debug)]
enum ReservoirError {
  SQLError,
  PoolClosed,
  ParseError,
  ColumnNotFound(String),
}

impl fmt::Display for ReservoirError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      ReservoirError::SQLError => write!(f, "SQLError"),
      ReservoirError::PoolClosed => write!(f, "PoolClosed"),
      ReservoirError::ParseError => write!(f, "ParseError"),
      ReservoirError::ColumnNotFound(s) => write!(f, "ColumnNotFound {}", s), 
    }
  }
}

impl From<sqlx::Error> for ReservoirError {
  fn from(e: sqlx::Error) -> Self {
    match e {
      sqlx::Error::PoolClosed => ReservoirError::PoolClosed,
      sqlx::Error::ColumnNotFound(s) => ReservoirError::ColumnNotFound(s),
      _ => ReservoirError::SQLError,
    }

  }
}

/*
impl Error for ReservoirError {
  fn description(&self) -> &str {
    use ReservoirError::*;
    match *self {
      SQLError => "Error on SQL",
      PoolClosed => "SQL connection pool closed",
      ParseError => "Query parse error",
      ColumnNotFound(_) => "Column not found",
    }
  }
}
*/