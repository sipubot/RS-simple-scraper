use serde::{Serialize, Deserialize};

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
