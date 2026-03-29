use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct List {
    pub timestamp: i64,
    pub title: String,
    pub datetime: String,
    pub link: String,
    pub images: String,
    pub more: String,
    pub new: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Site {
    pub host: String,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Save {
    pub host: String,
    pub json_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Down {
    pub host: String,
    pub title: String,
    pub path: String,
    #[serde(default)]
    pub use_webdriver: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Images {
    pub link: String,
    pub refferer: String,
    pub file_name: String,
    pub path: String,
    pub subpath: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Nick {
    pub nick: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub sites: Vec<Site>,
    #[serde(default)]
    pub saves: Vec<Save>,
    #[serde(default)]
    pub downs: Vec<Down>,
    #[serde(default)]
    pub nicks: Vec<Nick>,
}
