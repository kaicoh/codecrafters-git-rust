use std::time::{SystemTime, UNIX_EPOCH};

const AUTHOR_NAME: &str = "Kanji Tanaka";
const AUTHOR_EMAIL: &str = "sumireminami@gmail.com";

#[derive(Debug, Clone, PartialEq)]
pub struct Commit {
    tree: String,
    parent: Option<String>,
    comment: String,
}

impl Commit {
    pub fn new(tree: String, comment: String, parent: Option<String>) -> Self {
        Self {
            tree,
            comment,
            parent,
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let tree_sha = self.tree.as_str();
        let parents = if let Some(ref parent) = self.parent {
            format!("parent {parent}\n")
        } else {
            "".to_string()
        };
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let comment = self.comment.as_str();
        format!("tree {tree_sha}\n{parents}author {AUTHOR_NAME} <{AUTHOR_EMAIL}> {now} +0000\ncommitter {AUTHOR_NAME} <{AUTHOR_EMAIL}> {now} +0000\n\n{comment}\n")
            .as_bytes()
            .to_vec()
    }

    pub fn len(&self) -> usize {
        self.serialize().len()
    }
}
