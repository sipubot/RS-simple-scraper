use scraper::{Html, Selector};
use chrono::Utc;
use chrono_tz::Asia::Seoul;
use anyhow::Result;
use crate::models::List;

pub fn parse_fm(html: &str) -> Result<Vec<List>> {
    let mut _list: Vec<List> = vec![];
    let _today = Utc::now().with_timezone(&Seoul);
    let fragment = Html::parse_fragment(html);
    let part_sel = Selector::parse("div.li").map_err(|_| anyhow::anyhow!("Invalid fm selector"))?;
    let a_sel = Selector::parse("h3.title > a").map_err(|_| anyhow::anyhow!("Invalid fm a selector"))?;
    
    for element in fragment.select(&part_sel) {
        for _li in element.select(&a_sel) {
            let _title = _li.inner_html();
            let _link = _li.value().attr("href").unwrap_or_default();

            _list.push(List {
                timestamp: _today.timestamp(),
                title: _title,
                datetime: String::new(),
                link: _link.to_string(),
                images: String::new(),
                more: "펨코".to_string(),
                new: true,
            });
        }
    }
    Ok(_list)
}
