use std::path::Path;
use std::collections::HashSet;
use chrono::Utc;
use chrono_tz::Asia::Seoul;
use url::Url;
use futures::future::join_all;
use anyhow::{Result, Context};

mod utils;
mod foxfox;
mod models;
mod scrapers;

use models::{List, Site, Save, Down, Images, Nick};
use scrapers::{dc, fm, mp};

const SAVE_PATH: &str = "./save.json";
const SITE_PATH: &str = "./site.json";
const DOWN_PATH: &str = "./down.json";
const NICK_RULE: &str = "./nick.json";

async fn scrape_site(site: Site) -> Vec<List> {
    match site.host.as_str() {
        "dc" => {
            let html = utils::get_text_response(&site.url).await;
            if !html.is_empty() {
                let nick_json = utils::file_read_to_json(NICK_RULE).await.unwrap_or_default();
                let nick_list: Vec<Nick> = serde_json::from_value(nick_json).unwrap_or_default();
                
                let (results, logs) = dc::parse_dc(&html, &site.url, &nick_list);
                for log_link in logs {
                    utils::logger(&log_link).await;
                }
                results.unwrap_or_default()
            } else {
                vec![]
            }
        },
        "fm" => {
            let html = utils::get_text_response(&site.url).await;
            if !html.is_empty() {
                fm::parse_fm(&html).unwrap_or_default()
            } else {
                vec![]
            }
        },
        "mp" => {
            let html = utils::get_text_response_bot(&site.url).await;
            if !html.is_empty() {
                mp::parse_mp(&html).unwrap_or_default()
            } else {
                vec![]
            }
        },
        "mp_low" => {
            let html = utils::get_text_response_bot(&site.url).await;
            if !html.is_empty() {
                mp::parse_mp_part_low(&html).unwrap_or_default()
            } else {
                vec![]
            }
        },
        _ => {
            println!("not matched site: {}", site.host);
            utils::logger(&format!("not matched site: {}", site.host)).await;
            vec![]
        }
    }
}

async fn run_scraping_cycle() -> Result<()> {
    let site_json = utils::file_read_to_json(SITE_PATH).await.unwrap_or_default();
    let site_list: Vec<Site> = serde_json::from_value(site_json).unwrap_or_default();
    
    let save_json = utils::file_read_to_json(SAVE_PATH).await.unwrap_or_default();
    let mut save_list: Vec<Save> = serde_json::from_value(save_json).unwrap_or_default();

    let mut dc_list: Vec<List> = vec![];
    let mut fm_list: Vec<List> = vec![];
    let mut mp_list: Vec<List> = vec![];

    let mut dc_down_list: Vec<List> = vec![];
    let mut down_image_list = vec![];

    let scrape_tasks: Vec<_> = site_list.into_iter().enumerate().map(|(i, site)| {
        tokio::spawn(async move {
            if i > 0 {
                let delay = std::time::Duration::from_millis((i * 500) as u64);
                tokio::time::sleep(delay).await;
            }
            scrape_site(site).await
        })
    }).collect();

    let scrape_results: Vec<Vec<List>> = join_all(scrape_tasks).await
        .into_iter()
        .map(|r| r.unwrap_or_default())
        .collect();

    for posts in scrape_results {
        for post in posts {
            match post.more.as_str() {
                "디시" => dc_list.push(post),
                "펨코" => fm_list.push(post),
                "엠팍" => mp_list.push(post),
                _ => {}
            }
        }
    }

    for _save in save_list.iter_mut() {
        match _save.host.as_ref() {
            "dc" => {
                let _loadfile = load_file_to_list(&_save.json_path).await;
                dc_down_list = newer_to_list(&dc_list, &_loadfile);
                let merged = merge_to_list(&dc_list, &_loadfile);
                let save_json = serde_json::to_value(merged).context("Failed to serialize dc list")?;
                utils::file_save_from_json(&_save.json_path, &save_json).await?;
            },
            "fm" => {
                let _loadfile = load_file_to_list(&_save.json_path).await;
                let merged = merge_to_list(&fm_list, &_loadfile);
                let save_json = serde_json::to_value(merged).context("Failed to serialize fm list")?;
                utils::file_save_from_json(&_save.json_path, &save_json).await?;
            },
            "mp" => {
                let _loadfile = load_file_to_list(&_save.json_path).await;
                let merged = merge_to_list(&mp_list, &_loadfile);
                let save_json = serde_json::to_value(merged).context("Failed to serialize mp list")?;
                utils::file_save_from_json(&_save.json_path, &save_json).await?;
            },
            _ => {}
        }
    }

    for _downlink in dc_down_list.iter_mut() {
        let path = check_download(&_downlink.title).await;
        if !path.is_empty() {
            let ho_url = Url::parse(&_downlink.link).context("Failed to parse downlink URL")?;
            let host = format!("{}://{}", ho_url.scheme(), ho_url.host_str().unwrap_or_default());
            let html = foxfox::get_html(&_downlink.link).await.unwrap_or_default();
            let mut _list: Vec<Images> = dc::parse_dcimage(&html, &path, &_downlink.title, &host)?;
            down_image_list.append(&mut _list);
        }
    }

    for _down in down_image_list.iter_mut() {
        let data = utils::get_byte_response(&_down.link, &_down.refferer).await;
        if !data.is_empty() {
            let path = format!("{}/{}", &_down.path, &_down.subpath);
            let _ = utils::make_file(&path, &_down.file_name, &data).await;
        }
    }

    println!("End Of job");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(300));
    loop {
        interval.tick().await;
        if let Err(e) = run_scraping_cycle().await {
            utils::logger(&format!("Scraping cycle failed: {}", e)).await;
            eprintln!("Scraping cycle failed: {}", e);
        }
    }
}

async fn check_download(_title: &str) -> String {
    let down_json = utils::file_read_to_json(DOWN_PATH).await.unwrap_or_default();
    let down_list: Vec<Down> = serde_json::from_value(down_json).unwrap_or_default();
    
    for _downtarget in down_list {
        if _title.contains(&_downtarget.title) {
            return _downtarget.path;
        }
    }
    String::new()
}

async fn load_file_to_list(path: &str) -> Vec<List> {
    if Path::new(path).exists() {
        let load_json = utils::file_read_to_json(path).await.unwrap_or_default();
        let load_list: Vec<List> = serde_json::from_value(load_json).unwrap_or_default();
        let _stamp = Utc::now().with_timezone(&Seoul).timestamp();
        
        load_list.into_iter()
            .map(|mut x| {
                if _stamp - x.timestamp > 28800 {
                    x.new = false;
                }
                x
            })
            .filter(|x| (_stamp - x.timestamp) < 172800)
            .collect()
    } else {
        vec![]
    }
}

fn newer_to_list(a: &[List], b: &[List]) -> Vec<List> {
    let existing_links: HashSet<&str> = b.iter()
        .map(|item| item.link.as_str())
        .collect();

    a.iter()
        .filter(|item| !existing_links.contains(item.link.as_str()))
        .cloned()
        .collect()
}

fn merge_to_list(a: &[List], b: &[List]) -> Vec<List> {
    let mut result = Vec::new();
    let mut seen_links = HashSet::new();

    for item in a {
        if seen_links.insert(item.link.as_str()) {
            result.push(item.clone());
        }
    }

    for item in b {
        if seen_links.insert(item.link.as_str()) {
            result.push(item.clone());
        }
    }

    result.sort_by(|x, y| y.timestamp.cmp(&x.timestamp));
    result
}
