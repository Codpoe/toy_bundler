use std::path::PathBuf;

pub fn fulfill_root_prefix(path: &str, root: &str) -> String {
  if path.starts_with("./") {
    PathBuf::from(root).join(path).to_string_lossy().to_string()
  } else {
    path.to_string()
  }
}
