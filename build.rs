use std::path::Path;

fn main() {
  let exists = Path::new("db.db").try_exists();
  if !matches!(exists, Ok(true)) {
    let _ = std::fs::copy("schema.db", "db.db");
  }
}
