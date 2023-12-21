#[derive(serde::Deserialize, Clone, Debug)]
pub struct Config {
    pub redis_url: String,
    pub database_url: String,
    #[serde(default = "defaults::asset_dir")]
    pub asset_dir: String,
    #[serde(default = "defaults::root_url")]
    pub root_url: String,
    pub user_content_url: String,
    pub s3_bucket_name: String,
    #[serde(default = "defaults::s3_region")]
    pub s3_region: Option<String>,
    pub s3_endpoint: Option<String>,
    pub s3_access_key: Option<String>,
    pub s3_secret_key: Option<String>,
    #[serde(default = "defaults::path_style")]
    pub s3_path_style: bool,
    pub r2_account_id: Option<String>,
    #[serde(default = "defaults::port")]
    pub port: u16,
}

mod defaults {
    pub(super) fn root_url() -> String {
        String::from("http://localhost:8080")
    }

    pub(super) fn asset_dir() -> String {
        String::from("./assets/public/")
    }

    pub(super) fn s3_region() -> Option<String> {
        Some(String::from("us-east-1"))
    }

    pub(super) fn port() -> u16 {
        8080
    }

    pub(super) fn path_style() -> bool {
        true
    }
}

impl Config {
    pub fn from_env() -> Self {
        let config: Config = envy::from_env().expect("Failed to read config");
        let root_url = config.root_url.trim_end_matches('/').to_string();
        let user_content_url = config.user_content_url.trim_end_matches('/').to_string();
        Self {
            root_url,
            user_content_url,
            ..config
        }
    }

    #[cfg(test)]
    pub fn test() -> Self {
        let test_db_num =
            crate::test::util::TEST_DB_NUM.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let s3_bucket_name = format!("test-bucket-{test_db_num}");
        Self {
            redis_url: format!("redis://redis/{test_db_num}"),
            database_url: "postgres://spd:spd@postgres/spd".to_string(),
            asset_dir: "./assets/public/".to_string(),
            root_url: "http://localhost:8080".to_string(),
            user_content_url: "http://localhost:8000".to_string(),
            s3_bucket_name,
            s3_region: Some("us-east-1".to_string()),
            s3_endpoint: Some("http://minio:9000".to_string()),
            s3_access_key: Some("us-east-1".to_string()),
            s3_secret_key: Some("us-east-1".to_string()),
            s3_path_style: true,
            r2_account_id: None,
            port: 8080,
        }
    }
}
