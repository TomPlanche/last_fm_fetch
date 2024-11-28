use std::collections::HashMap;

pub type QueryParams = HashMap<String, String>;

#[derive(Debug, Clone)]
pub struct Url {
    base: String,
    query_params: QueryParams,
}

impl Url {
    pub fn new(base: &str) -> Self {
        Url {
            base: base.to_string(),
            query_params: HashMap::new(),
        }
    }

    pub fn add_args(mut self, args: QueryParams) -> Self {
        self.query_params.extend(args);

        self
    }

    pub fn build(&self) -> String {
        if self.query_params.is_empty() {
            return self.base.clone();
        }

        let query_string: Vec<String> = self
            .query_params
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect();

        format!("{}?{}", self.base, query_string.join("&"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_url() {
        let url = Url::new("https://www.google.com");
        assert_eq!(url.build(), "https://www.google.com");
    }

    #[test]
    fn test_single_query_param() {
        let mut params = HashMap::new();
        params.insert("q".to_string(), "rust".to_string());

        let url = Url::new("https://www.google.com").add_args(params);
        assert_eq!(url.build(), "https://www.google.com?q=rust");
    }

    #[test]
    fn test_multiple_query_params() {
        let mut params = HashMap::new();
        params.insert("q".to_string(), "rust".to_string());
        params.insert("lang".to_string(), "en".to_string());

        let url = Url::new("https://www.google.com").add_args(params);
        let built_url = url.build();
        assert!(
            built_url == "https://www.google.com?q=rust&lang=en"
                || built_url == "https://www.google.com?lang=en&q=rust"
        );
    }

    #[test]
    fn test_chained_param_addition() {
        let url = Url::new("https://www.example.com")
            .add_args(HashMap::from([("page".to_string(), "1".to_string())]))
            .add_args(HashMap::from([("limit".to_string(), "10".to_string())]));

        let built_url = url.build();
        assert!(
            built_url == "https://www.example.com?page=1&limit=10"
                || built_url == "https://www.example.com?limit=10&page=1"
        );
    }

    #[test]
    fn test_param_overwrite() {
        let mut params1 = HashMap::new();
        params1.insert("key".to_string(), "value1".to_string());

        let mut params2 = HashMap::new();
        params2.insert("key".to_string(), "value2".to_string());

        let url = Url::new("https://www.example.com")
            .add_args(params1)
            .add_args(params2);

        assert_eq!(url.build(), "https://www.example.com?key=value2");
    }
}
