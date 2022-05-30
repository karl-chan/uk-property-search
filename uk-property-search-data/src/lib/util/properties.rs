use config::{Config, File};

pub struct Properties {
    config: Config,
}

impl Properties {
    pub fn new() -> Self {
        Self {
            config: Config::builder()
                .add_source(File::with_name("properties.toml"))
                .build()
                .unwrap(),
        }
    }

    pub fn get_int(&self, key: &str) -> i64 {
        self.config.get_int(key).unwrap()
    }

    pub fn get_string(&self, key: &str) -> String {
        self.config.get_string(key).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_int() {
        assert_eq!(Properties::new().get_int("server.default.port"), 3000)
    }
}
