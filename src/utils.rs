use sha2::{Sha256, Digest};

pub fn get_db_url() -> String {
  "./db/reservoir.db".to_string()
  // fs::read_to_string("sqlinfo.config")
  //    .expect("Failed to load setting")
}

pub fn hash(input: &Option<String>) -> String {
  let mut hashed = Sha256::new();
  let string = match input {
    Some(s) => s,
    None => "",
  };
  hashed.update(string);
  format!("{:X}", hashed.finalize())
}