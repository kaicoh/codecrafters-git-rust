use regex::Regex;
use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq)]
struct User {
    name: String,
    email: String,
    timestamp: u64,
    timezone: String,
}

impl From<&[u8]> for User {
    fn from(bytes: &[u8]) -> Self {
        let re =
            Regex::new(r"(?<name>.+) <(?<email>.+)> (?<timestamp>\d+) (?<timezone>.+)").unwrap();
        let st = stringify(bytes);
        let caps = re.captures(st.as_str()).unwrap();
        Self {
            name: (caps["name"]).to_string(),
            email: (caps["email"]).to_string(),
            timestamp: (caps["timestamp"]).parse().unwrap(),
            timezone: (caps["timezone"]).to_string(),
        }
    }
}

impl Default for User {
    fn default() -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        Self {
            name: "Kanji Tanaka".into(),
            email: "sumireminami@gmail.com".into(),
            timestamp,
            timezone: "+0000".to_string(),
        }
    }
}

impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} <{}> {} {}",
            self.name, self.email, self.timestamp, self.timezone
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Commit {
    tree: String,
    parents: Vec<String>,
    comment: String,
    author: User,
    committer: User,
}

impl Commit {
    pub fn new(tree: String, comment: String, parents: Vec<String>) -> Self {
        Self {
            tree,
            comment,
            parents,
            author: User::default(),
            committer: User::default(),
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let parents: String = self
            .parents
            .iter()
            .map(|p| format!("parent {p}\n"))
            .collect::<Vec<String>>()
            .join("");
        [
            format!("tree {}\n", self.tree),
            parents,
            format!("author {}\n", self.author),
            format!("committer {}\n", self.committer),
            format!("\n{}\n", self.comment),
        ]
        .join("")
        .into_bytes()
    }

    pub fn len(&self) -> usize {
        self.serialize().len()
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut parents: Vec<String> = vec![];

        let mut lines = bytes.split(|&b| b == b'\n');
        let mut line = lines.next().unwrap();
        let tree = stringify(&line[5..]);

        line = lines.next().unwrap();
        while line.starts_with(b"parent") {
            parents.push(stringify(&line[7..]));
            line = lines.next().unwrap();
        }

        let author = User::from(&line[7..]);

        line = lines.next().unwrap();
        let committer = User::from(&line[10..]);

        lines.next().unwrap();
        line = lines.next().unwrap();
        let comment = Some(stringify(line));

        Self {
            tree,
            parents,
            comment: comment.unwrap(),
            author,
            committer,
        }
    }
}

fn stringify(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_creates_user() {
        let bytes: &[u8] = b"Kanji Tanaka <sumireminami@gmail.com> 946684800 +0000";
        let user = User::from(bytes);
        assert_eq!(
            user,
            User {
                name: "Kanji Tanaka".into(),
                email: "sumireminami@gmail.com".into(),
                timestamp: 946684800,
                timezone: "+0000".into(),
            }
        );
    }

    #[test]
    fn it_creates_commit_from_bytes() {
        let bytes: &[u8] = b"tree 8119b90c6adef211483e6dcf1a3c89e966af9c60\nparent b521b9179412d90a893bc36f33f5dcfd987105ef\nauthor Paul Kuruvilla <rohitpaulk@gmail.com> 1587032850 +0530\ncommitter Paul Kuruvilla <rohitpaulk@gmail.com> 1587032850 +0530\n\nUpdate content\n";
        let user = User {
            name: "Paul Kuruvilla".into(),
            email: "rohitpaulk@gmail.com".into(),
            timestamp: 1587032850,
            timezone: "+0530".into(),
        };
        let commit = Commit {
            tree: "8119b90c6adef211483e6dcf1a3c89e966af9c60".into(),
            parents: vec!["b521b9179412d90a893bc36f33f5dcfd987105ef".into()],
            comment: "Update content".into(),
            author: user.clone(),
            committer: user,
        };
        assert_eq!(Commit::from_bytes(bytes), commit);
    }
}
