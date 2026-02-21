use scraper::{Html, Selector};
use chrono::Utc;
use chrono_tz::Asia::Seoul;
use anyhow::Result;
use crate::models::List;

pub fn parse_mp(html: &str) -> Result<Vec<List>> {
    let mut _list: Vec<List> = vec![];
    let fragment = Html::parse_fragment(html);
    let _today = Utc::now().with_timezone(&Seoul);
    let table_sel = Selector::parse("table.tbl_type01").map_err(|_| anyhow::anyhow!("Invalid mp table selector"))?;
    let tr_sel = Selector::parse("tbody > tr").map_err(|_| anyhow::anyhow!("Invalid mp tr selector"))?;
    let a_sel = Selector::parse("td.t_left > a").map_err(|_| anyhow::anyhow!("Invalid mp a selector"))?;
    let date_sel = Selector::parse("td > span.date").map_err(|_| anyhow::anyhow!("Invalid mp date selector"))?;

    for _table in fragment.select(&table_sel) {
        for _tr in _table.select(&tr_sel) {
            let _a = match _tr.select(&a_sel).next() {
                Some(v) => v,
                None => continue,
            };
            let _date = match _tr.select(&date_sel).next() {
                Some(v) => v,
                None => continue,
            };
            
            let _link = _a.value().attr("href").unwrap_or_default();
            let _title = _a.inner_html();
            
            _list.push(List {
                timestamp: _today.timestamp(),
                title: _title,
                datetime: _date.inner_html(),
                link: _link.to_string(),
                images: String::new(),
                more: "엠팍".to_string(),
                new: true,
            });
        }
    }
    Ok(_list)
}

pub fn parse_mp_part_low(html: &str) -> Result<Vec<List>> {
    let mut _list: Vec<List> = vec![];
    let _today = Utc::now().with_timezone(&Seoul);
    let fragment = Html::parse_fragment(html);

    let div_sel = Selector::parse("div.lists_today_contxt").map_err(|_| anyhow::anyhow!("Invalid mp low div selector"))?;
    let li_sel = Selector::parse("li.items").map_err(|_| anyhow::anyhow!("Invalid mp low li selector"))?;
    let a_sel = Selector::parse("a").map_err(|_| anyhow::anyhow!("Invalid mp low a selector"))?;
    
    for _div in fragment.select(&div_sel) {
        for _li in _div.select(&li_sel) {
            let _a = match _li.select(&a_sel).next() {
                Some(v) => v,
                None => continue,
            };
            let _link = _a.value().attr("href").unwrap_or_default();
            let _title = _a.inner_html();
            
            _list.push(List {
                timestamp: _today.timestamp(),
                title: _title,
                datetime: String::new(),
                link: _link.to_string(),
                images: String::new(),
                more: "엠팍".to_string(),
                new: true,
            });
        }
    }
    Ok(_list)
}
