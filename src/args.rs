use std::collections::HashMap;

#[derive(Debug, Default)]
pub(crate) struct ArgsBuilder {
    positions: Vec<(usize, String)>,
    flags: Vec<String>,
    single_args: Vec<String>,
}

impl ArgsBuilder {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn position(mut self, pos: usize, name: &str) -> Self {
        self.positions.push((pos, name.into()));
        self
    }

    pub(crate) fn flag(mut self, name: &str) -> Self {
        self.flags.push(name.into());
        self
    }

    pub(crate) fn arg(mut self, name: &str) -> Self {
        self.single_args.push(name.into());
        self
    }

    pub(crate) fn build(self, args: &[String]) -> Args {
        let mut map: HashMap<String, ArgValue> = HashMap::new();
        let mut args = args.to_vec();
        let Self {
            mut positions,
            flags,
            single_args,
        } = self;

        for flag in flags {
            if let Some(pos) = args.iter().position(|v| v.as_str() == flag.as_str()) {
                args.remove(pos);
                map.insert(flag, ArgValue::Bool(true));
            }
        }

        for single_arg in single_args {
            if let Some(pos) = args.iter().position(|v| v.as_str() == single_arg.as_str()) {
                args.remove(pos);
                let value = args.remove(pos);
                map.insert(single_arg, ArgValue::String(value));
            }
        }

        positions.sort_by(|a, b| a.0.cmp(&b.0));
        for (pos, name) in positions {
            if let Some(value) = args.get(pos) {
                map.insert(name, ArgValue::String(value.into()));
            }
        }

        Args(map)
    }
}

#[derive(Debug)]
enum ArgValue {
    Bool(bool),
    String(String),
}

#[derive(Debug)]
pub(crate) struct Args(HashMap<String, ArgValue>);

impl Args {
    pub(crate) fn builder() -> ArgsBuilder {
        ArgsBuilder::new()
    }

    pub(crate) fn flag(&self, key: &str) -> bool {
        match self.0.get(key) {
            Some(ArgValue::Bool(v)) => *v,
            _ => false,
        }
    }

    pub(crate) fn value(&self, key: &str) -> Option<String> {
        match self.0.get(key) {
            Some(ArgValue::String(v)) => Some(v.into()),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_parses_arg_with_single_values() {
        let values = vec!["-p".to_string(), "/foo/bar".to_string()];
        let args = Args::builder().arg("-p").build(&values);
        assert_eq!(args.value("-p"), Some("/foo/bar".into()));
        assert_eq!(args.value("no_key"), None);
    }

    #[test]
    fn it_parses_flag_arg() {
        let values = vec!["--foo".to_string()];
        let args = Args::builder().flag("--foo").flag("--bar").build(&values);
        assert!(args.flag("--foo"));
        assert!(!args.flag("--bar"));
    }

    #[test]
    fn it_parses_position_args() {
        let values = vec!["foo.csv".to_string()];
        let args = Args::builder()
            .position(0, "file")
            .position(1, "notfound")
            .build(&values);
        assert_eq!(args.value("file"), Some("foo.csv".into()));
        assert_eq!(args.value("notfound"), None);
    }

    #[test]
    fn it_parses_multiple_args() {
        let values = vec![
            "--foobar".to_string(),
            "-d".to_string(),
            "/barbaz".to_string(),
            "foobarbaz.csv".to_string(),
        ];
        let args = Args::builder()
            .flag("--foobar")
            .arg("-d")
            .position(0, "file")
            .build(&values);
        assert!(args.flag("--foobar"));
        assert_eq!(args.value("-d"), Some("/barbaz".into()));
        assert_eq!(args.value("file"), Some("foobarbaz.csv".into()));
    }

    #[test]
    fn it_parses_multiple_positional_args() {
        let values = vec!["foobar".to_string(), "foobarbaz".to_string()];
        let args = Args::builder()
            .position(0, "url")
            .position(1, "dir")
            .build(&values);
        assert_eq!(args.value("url"), Some("foobar".into()));
        assert_eq!(args.value("dir"), Some("foobarbaz".into()));
    }
}
