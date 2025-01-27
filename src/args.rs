use std::collections::HashMap;

#[derive(Debug)]
pub(crate) struct Args(HashMap<String, Vec<String>>);

impl Args {
    pub(crate) fn new(args: &[String]) -> Self {
        let mut hashmap: HashMap<String, Vec<String>> = HashMap::new();
        let mut key: Option<String> = None;

        for token in args {
            if token.as_str().starts_with("-") {
                key = Some(token[1..].into());
            } else if let Some(k) = key.as_ref() {
                match hashmap.get_mut(k) {
                    Some(values) => {
                        values.push(token.into());
                    }
                    None => {
                        hashmap.insert(k.into(), vec![token.into()]);
                    }
                }
            }
        }
        Self(hashmap)
    }

    pub(crate) fn value(&self, key: &str) -> Option<String> {
        self.0.get(key).and_then(|v| v.iter().next()).cloned()
    }

    #[allow(dead_code)]
    pub(crate) fn values(&self, key: &str) -> Vec<String> {
        self.0.get(key).map(|v| v.to_vec()).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_parses_arg_with_single_values() {
        let args = vec!["-p".to_string(), "/foo/bar".to_string()];
        let args = Args::new(&args);
        assert_eq!(args.value("p"), Some("/foo/bar".into()));
        assert_eq!(args.value("no_key"), None);
    }

    #[test]
    fn it_parses_arg_with_multiple_values() {
        let args = vec!["-p".to_string(), "/foo".to_string(), "/bar".to_string()];
        let args = Args::new(&args);
        assert_eq!(
            args.values("p"),
            vec!["/foo".to_string(), "/bar".to_string()]
        );
        assert_eq!(args.values("no_key"), Vec::<String>::new());
    }

    #[test]
    fn it_parses_multiple_args() {
        let args = vec![
            "-p".to_string(),
            "/foo".to_string(),
            "-f".to_string(),
            "foo.csv".to_string(),
            "bar.csv".to_string(),
        ];
        let args = Args::new(&args);
        assert_eq!(args.value("p"), Some("/foo".into()));
        assert_eq!(
            args.values("f"),
            vec!["foo.csv".to_string(), "bar.csv".to_string()]
        );
    }
}
