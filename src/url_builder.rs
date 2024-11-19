#[derive(Debug, Clone)]
pub struct Url {
    base: String,
    query_params: Vec<(String, String)>,
}

impl Url {
    pub fn new(base: &str) -> Self {
        Url {
            base: base.to_string(),
            query_params: Vec::new(),
        }
    }

    pub fn add_args(mut self, args: Vec<(&str, &str)>) -> Self {
        self.query_params.extend(
            args.into_iter()
                .map(|(k, v)| (k.to_string(), v.to_string())),
        );
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
