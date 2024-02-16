use std::path::PathBuf;

pub fn stripe_root_prefix(path: &str) -> String {
  if path.starts_with("root:") {
    path.strip_prefix("root:").unwrap().to_string()
  } else {
    path.to_string()
  }
}

pub fn fulfill_root_prefix(path: &str, root: &str) -> String {
  if path.starts_with("root:") {
    PathBuf::from(root)
      .join(path.strip_prefix("root:").unwrap())
      .to_string_lossy()
      .to_string()
  } else {
    path.to_string()
  }
}
