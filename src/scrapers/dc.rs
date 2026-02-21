use scraper::{Html, Selector};
use chrono::Utc;
use chrono_tz::Asia::Seoul;
use url::Url;
use anyhow::{Result, Context};
use crate::models::{List, Nick, Images};

pub fn parse_dc(html: &str, site_url: &str, nick_list: &[Nick]) -> (Result<Vec<List>>, Vec<String>) {
    let mut _list: Vec<List> = vec![];
    let mut logs = Vec::new();
    let _today = Utc::now().with_timezone(&Seoul);
    let fragment = Html::parse_fragment(html);
    
    let meta_link_sel_res = Selector::parse(r#"input[id="list_url"]"#);
    let part_sel_res = Selector::parse("tr.ub-content");
    let title_sel_res = Selector::parse("td.gall_tit > a");
    let date_sel_res = Selector::parse("td.gall_date");
    let nick_sel_res = Selector::parse("td.gall_writer");

    if let (Ok(meta_link_sel), Ok(part_sel), Ok(title_sel), Ok(date_sel), Ok(nick_sel)) = 
           (meta_link_sel_res, part_sel_res, title_sel_res, date_sel_res, nick_sel_res) {
        
        let _host_val = fragment.select(&meta_link_sel).next()
            .and_then(|v| v.value().attr("value"))
            .unwrap_or_default();

        let host_str = if !_host_val.is_empty() {
            Url::parse(_host_val).ok()
                .and_then(|u| u.host_str().map(|h| h.to_string()))
                .unwrap_or_else(|| Url::parse(site_url).ok()
                    .and_then(|u| u.host_str().map(|h| h.to_string()))
                    .unwrap_or_default())
        } else {
            Url::parse(site_url).ok()
                .and_then(|u| u.host_str().map(|h| h.to_string()))
                .unwrap_or_default()
        };

        for element in fragment.select(&part_sel) {
            let td_title = match element.select(&title_sel).next() {
                Some(v) => v,
                None => continue,
            };
            
            let _title_raw = td_title.inner_html();
            let _link = td_title.value().attr("href").unwrap_or_default().to_string();
            
            let td_date = match element.select(&date_sel).next() {
                Some(v) => v,
                None => continue,
            };
            let _date = td_date.value().attr("title").unwrap_or_default().to_string();
            let _date_text = td_date.inner_html();
            
            let _nick_text = element.select(&nick_sel).next()
                .and_then(|v| v.value().attr("data-nick"))
                .unwrap_or_default()
                .to_string();

            let _timestamp = chrono::NaiveDateTime::parse_from_str(&_date, "%Y-%m-%d %H:%M:%S");

            let processed_title = _title_raw.split("</em>")
                .last()
                .unwrap_or("")
                .chars()
                .filter(|&c| c != '\n' && c != '\t')
                .collect::<String>();

            if let Ok(v) = _timestamp {
                let _diff = _today.timestamp() - v.and_utc().timestamp();
                if _diff < 172800 && !nick_list.iter().any(|e| _nick_text == e.nick) {
                    _list.push(List {
                        timestamp: _today.timestamp(),
                        title: processed_title,
                        datetime: _date_text,
                        link: format!("https://{}{}", host_str, _link),
                        images: String::new(),
                        more: "디시".to_string(),
                        new: true,
                    });
                }
            } else {
                logs.push(_link);
            }
        }
        (Ok(_list), logs)
    } else {
        (Err(anyhow::anyhow!("Invalid selectors")), Vec::new())
    }
}

pub fn parse_dcimage(html: &str, path: &str, title: &str, host: &str) -> Result<Vec<Images>> {
    let mut nums = 1;
    let mut _list: Vec<Images> = vec![];
    let fragment = Html::parse_fragment(html);
    let images_selector = Selector::parse(r#"img"#).map_err(|_| anyhow::anyhow!("Invalid img selector"))?;

    let tag_regex = regex::Regex::new(r"<.*?>").context("Failed to compile tag regex")?;
    let mut _title = tag_regex.replace_all(title, "").to_string();
    
    let illegal_chars = ['/', '\\', ':', '*', '?', '"', '>', '<', '|'];
    _title = _title.chars()
        .map(|c| if illegal_chars.contains(&c) { '_' } else { c })
        .collect();

    for element in fragment.select(&images_selector) {
        let url = element.value().attr("src").unwrap_or_default();
        let ev = element.value().attr("onerror").unwrap_or_default();
        if ev.contains("reload_img(this)") {
            _list.push(Images {
                link: url.to_string(),
                refferer: host.to_string(),
                file_name: format!("{}.jpg", nums),
                path: path.to_string(),
                subpath: _title.clone(),
            });
            nums += 1;
        }
    }
    Ok(_list)
}
