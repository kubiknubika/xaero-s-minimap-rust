//! `XaeroPath`: the path type the minimap uses for containers, worlds and
//! sub-containers (mirrors `xaero/hud/path/XaeroPath`).

use std::fmt;
use std::path::PathBuf;

/// A chain of directory names from the minimap root down to a node.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct XaeroPath {
    parts: Vec<String>,
}

impl XaeroPath {
    pub fn new() -> Self {
        XaeroPath { parts: Vec::new() }
    }

    /// Path built from parts.
    pub fn of(parts: &[&str]) -> Self {
        XaeroPath {
            parts: parts.iter().map(|s| (*s).to_string()).collect(),
        }
    }

    /// Appends a part.
    pub fn push(&mut self, part: &str) {
        self.parts.push(part.to_string());
    }

    /// Removes and returns the last part.
    pub fn pop(&mut self) -> Option<String> {
        self.parts.pop()
    }

    pub fn is_empty(&self) -> bool {
        self.parts.is_empty()
    }

    /// Number of parts.
    pub fn len(&self) -> usize {
        self.parts.len()
    }

    pub fn parts(&self) -> &[String] {
        &self.parts
    }

    /// Last part of the path (the node's own name).
    pub fn file_name(&self) -> Option<&str> {
        self.parts.last().map(|s| s.as_str())
    }

    /// Copy of the path with `part` appended.
    pub fn join(&self, part: &str) -> XaeroPath {
        let mut out = self.clone();
        out.push(part);
        out
    }

    /// Path to a `PathBuf` for a base directory (all parts).
    pub fn to_path_buf(&self) -> PathBuf {
        self.parts.iter().collect::<PathBuf>()
    }

    /// True if `prefix` is a leading part of `self`.
    pub fn starts_with(&self, prefix: &XaeroPath) -> bool {
        self.parts.starts_with(&prefix.parts)
    }
}

impl FromIterator<String> for XaeroPath {
    fn from_iter<I: IntoIterator<Item = String>>(iter: I) -> Self {
        XaeroPath {
            parts: iter.into_iter().collect(),
        }
    }
}

impl fmt::Display for XaeroPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.parts.join("/"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_push_pop_and_names() {
        let mut p = XaeroPath::new();
        p.push("a");
        p.push("b");
        assert_eq!(p.to_string(), "a/b");
        assert_eq!(p.file_name(), Some("b"));
        assert_eq!(p.len(), 2);
        assert!(!p.is_empty());
        assert_eq!(p.pop(), Some("b".to_string()));
        assert_eq!(p.file_name(), Some("a"));
    }

    #[test]
    fn starts_with_prefix() {
        let base = XaeroPath::of(&["root"]);
        let child = base.join("worlds");
        assert!(child.starts_with(&base));
        assert!(!base.starts_with(&child));
    }
}
