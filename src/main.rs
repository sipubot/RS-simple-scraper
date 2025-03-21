#[macro_use]
extern crate serde_derive;

use std::path::Path;
use std::collections::HashSet;
use scraper::{Html, Selector};
use chrono::Utc;
use chrono_tz::Asia::Seoul;
use url::Url;
mod utils;

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

const SAVE_PATH: &str = "./save.json";
const SITE_PATH: &str = "./site.json";
const DOWN_PATH: &str = "./down.json";
const NICK_RULE: &str = "./nick.json";

#[tokio::main]
async fn main() -> std::io::Result<()> {

    let mut site_list : Vec<Site> = serde_json::from_value(utils::file_read_to_json(SITE_PATH).unwrap_or_default()).unwrap_or_default();
    let mut save_list : Vec<Save> = serde_json::from_value(utils::file_read_to_json(SAVE_PATH).unwrap_or_default()).unwrap_or_default();

    let __loop = tokio::task::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(300));
        loop {
            interval.tick().await;
        
            let mut dc_list: Vec<List>  = vec![];
            let mut fm_list: Vec<List>  = vec![];
            let mut mp_list: Vec<List>  = vec![];
            
            let mut dc_down_list: Vec<List>  = vec![];
            let mut down_image_list = vec![];

            for _site in site_list.iter_mut() {
                match _site.host.as_ref() {
                    "dc" => {
                        let html = utils::get_text_response(&_site.url).await;
                        if html.len() > 0 {
                            let mut _dc_list: Vec<List> = parse_dc(&html);
                            dc_list.append(&mut _dc_list);
                        }
                    },
                    "fm" => {
                        let html = utils::get_text_response(&_site.url).await;
                        if html.len() > 0 {
                            let mut _fm_list: Vec<List> = parse_fm(&html);
                            fm_list.append(&mut _fm_list);
                        }
                    },
                    "mp" => {
                        let html = utils::get_text_response_bot(&_site.url).await;
                        if html.len() > 0 {
                            let mut _mp_list: Vec<List> = parse_mp(&html);
                            mp_list.append(&mut _mp_list);
                        }
                    },
                    "mp_low" => {
                        let html = utils::get_text_response_bot(&_site.url).await;
                        if html.len() > 0 {
                            let mut _mp_list: Vec<List> = parse_mp_part_low(&html);
                            mp_list.append(&mut _mp_list);
                        }
                        
                    },
                    _ => {
                        println!("not matched");
                        utils::logger("not matched site");
                    }
                }
            }
        
            for _save in save_list.iter_mut() {
                match _save.host.as_ref() {
                    "dc" => {
                        let mut _loadfile = load_file_to_list(&_save.json_path);
                        dc_down_list = newer_to_list(dc_list.to_vec(),_loadfile.to_vec());
                        let save_json = serde_json::to_value(merge_to_list(dc_list.to_vec(),_loadfile)).unwrap();
                        utils::file_save_from_json(&_save.json_path, &save_json).unwrap();
                    },
                    "fm" => {
                        let mut _loadfile = load_file_to_list(&_save.json_path);
                        let save_json = serde_json::to_value(merge_to_list(fm_list.to_vec(),_loadfile)).unwrap();
                        utils::file_save_from_json(&_save.json_path, &save_json).unwrap();
                                
                    },
                    "mp" => {
                        let mut _loadfile = load_file_to_list(&_save.json_path);
                        let save_json = serde_json::to_value(merge_to_list(mp_list.to_vec(),_loadfile)).unwrap();
                        utils::file_save_from_json(&_save.json_path, &save_json).unwrap();
                    },
                    "mp_low" => {
                        println!("skip");
                    },
                    _ => {
                        println!("not matched");
                        utils::logger("not matched site");
                    }
                }
            }

            for _downlink in dc_down_list.iter_mut() {
                let path = check_download(&_downlink.title);
                if path.len() > 0 {
                    let _url = &_downlink.link;
                    let ho_url = Url::parse(&_url).expect("REASON");
                    let host = format!("{}{}","https://",ho_url.host_str().unwrap());
                    let html = utils::get_text_response(&_url).await;
                    let mut _list: Vec<Images> = parse_dcimage(&html, &path, &_downlink.title, &host);
                    down_image_list.append(&mut _list);
                }
            }

            for _down in down_image_list.iter_mut() {
                let data = utils::get_byte_response(&_down.link, &_down.refferer).await;
                if data.len() > 0 {
                    let path = format!("{}/{}",&_down.path, &_down.subpath);
                    let _ = utils::make_file(&path, &_down.file_name, &data);
                }
            }
            println!("End Of job");    
            //git_push().await.unwrap_or_default();
        }
    });
    __loop.await?
}

fn check_download(_title : &str) -> String {
    let mut down_list : Vec<Down> = serde_json::from_value(utils::file_read_to_json(DOWN_PATH).unwrap_or_default()).unwrap_or_default();
    let mut _path = "".to_string();

    for _downtarget in down_list.iter_mut() {
        //println!("{:#?}", &_title); 
        if _title.contains(&_downtarget.title) {
            //println!("{:#?}", &_title);  
            _path = _downtarget.path.to_string();
        }
    }
    _path
}

fn parse_dcimage(html: &str, path: &str, title: &str, host: &str) -> Vec<Images> {
    let mut nums = 1;
    let mut _list: Vec<Images> = vec![];
    let fragment = Html::parse_fragment(&html);
    let images = Selector::parse(r#"img"#).unwrap();

    //title fix
    let tag = regex::Regex::new(r"<.*?>");
    let mut _title = tag.expect("REASON").replace_all(title, "");
    _title = _title.replace("/","").into();
    _title = _title.replace("\\","").into();
    _title = _title.replace(":","").into();
    _title = _title.replace("*","").into();
    _title = _title.replace("?","").into();
    _title = _title.replace("\"","").into();
    _title = _title.replace(">","").into();
    _title = _title.replace("<","").into();
    _title = _title.replace("|","").into();

    for element in fragment.select(&images) {
        //println!("{:#?}", element.value());    
        let url = element.value().attr("src").unwrap_or_default();
        let ev = element.value().attr("onerror").unwrap_or_default();
        if ev.contains("reload_img(this)") {
            //println!("{:#?}", &url);    
            _list.push(Images{
                link: url.to_string(),
                refferer : host.to_string(),
                file_name:format!("{}{}",nums,".jpg"),
                path:path.to_string(),
                subpath:_title.to_string(),
            });
            nums += 1;
        }
    }
    _list
}

#[warn(deprecated)]
fn parse_dc(html : &str) -> Vec<List> {
    let mut _list: Vec<List> = vec![];
    let _today = Utc::now().with_timezone(&Seoul);
    let fragment = Html::parse_fragment(html);
    let meta_link = Selector::parse(r#"input[id="list_url"]"#).unwrap();
    let part = Selector::parse("tr.ub-content").unwrap();
    let title = Selector::parse("td.gall_tit > a").unwrap();
    let date = Selector::parse("td.gall_date").unwrap();
    let nick = Selector::parse("td.gall_writer").unwrap();
    let _meta_value = fragment.select(&meta_link).next();
    let _host = match _meta_value {
        Some(v) => v.value().attr("value").unwrap_or_default(),
        None => ""
    };
    //존재하지 않는 페이지
    if _host.len() < 1 {
        return _list;
    }
    let ho_url = Url::parse(_host).expect("REASON");
    let host = ho_url.host_str().unwrap();

    let nick_list : Vec<Nick> = serde_json::from_value(utils::file_read_to_json(NICK_RULE).unwrap_or_default()).unwrap_or_default();

    
    for element in fragment.select(&part) {

        let td1 = element.select(&title).next().unwrap();
        let mut _title = td1.inner_html();
        let _link = td1.value().attr("href").unwrap_or_default();
        let _date = element.select(&date).next().unwrap().value().attr("title").unwrap_or_default();
        let _date_text = element.select(&date).next().unwrap().inner_html();
        let _nick_text = element.select(&nick).next().unwrap().value().attr("data-nick").unwrap_or_default();
        let _timestamp = chrono::NaiveDateTime::parse_from_str(_date,"%Y-%m-%d %H:%M:%S");
        _title = _title.split("</em>").last().unwrap().to_string().replace("\n", "").replace("\t", "").to_string();

        match _timestamp {
            Ok(v) => {
                //게시물 시간
                let _diff = _today.timestamp() - v.and_utc().timestamp();
                //println!("{:#?}, {:#?}, {:#?}", v.timestamp(), _diff, _title);
                if _diff < 172800 && !nick_list.iter().any(|e| _nick_text == e.nick) {
                    //println!("{:#?}, {:#?}, {:#?}", _title, _link, _date_text);
                    _list.push(List{
                        timestamp: _today.timestamp(),
                        title:_title,
                        datetime:_date_text,
                        link:format!("https://{}{}",host,_link.to_string()),
                        images:"".to_string(),
                        more:"디시".to_string(),
                        new: true,
                    })
                } 
            },
            Err(_) =>{
                //
                utils::logger(_link);
            },
        }
    }
    _list
}

fn parse_fm(html : &str) -> Vec<List> {
    let mut _list: Vec<List> = vec![];
    let _today = Utc::now().with_timezone(&Seoul);
    let fragment = Html::parse_fragment(html);
    let part = Selector::parse("div.li").unwrap();
    let a = Selector::parse("h3.title > a").unwrap();
    //let date = Selector::parse("td.gall_date").unwrap();
    
    for element in fragment.select(&part) {
        //println!("{:#?}", element.inner_html()); 
        for _li in element.select(&a){
            let _title = &_li.inner_html();
            let _link = &_li.value().attr("href").unwrap_or_default();

            _list.push(List{
                timestamp:_today.timestamp(),
                title:_title.to_string(),
                datetime:"".to_string(),
                link:_link.to_string(),
                images:"".to_string(),
                more:"펨코".to_string(),
                new: true,
            });
        }
    }
    _list
}

fn parse_mp(html : &str) -> Vec<List> {
    let mut _list: Vec<List> = vec![];
    let fragment = Html::parse_fragment(html);
    let _today = Utc::now().with_timezone(&Seoul);
    let table = Selector::parse("table.tbl_type01").unwrap();
    let tr = Selector::parse("tbody > tr").unwrap();
    let a = Selector::parse("td.t_left > a").unwrap();
    let date = Selector::parse("td > span.date").unwrap();
    //println!("{:#?}", fragment.select(&table).next().unwrap().inner_html());  
    for _table in fragment.select(&table) {
        //println!("{:#?}", _table.inner_html());  
        for _tr in _table.select(&tr) {
            //println!("{:#?}", _tr.inner_html());  
            let _a = _tr.select(&a).next().unwrap();
            let _date = _tr.select(&date).next().unwrap().inner_html();
            let _link = _a.value().attr("href").unwrap_or_default();
            let _title = _a.inner_html();
            //println!("{:#?}", _a.inner_html());  
            _list.push(List{
                timestamp:_today.timestamp(),
                title:_title,
                datetime:_date,
                link:_link.to_string(),
                images:"".to_string(),
                more:"엠팍".to_string(),
                new: true,
            });
        }

    }
    _list
}

fn parse_mp_part_low(html : &str) -> Vec<List> {
    let mut _list: Vec<List> = vec![];
    let _today = Utc::now().with_timezone(&Seoul);
    let fragment = Html::parse_fragment(html);

    let div = Selector::parse("div.lists_today_contxt").unwrap();
    let li = Selector::parse("li.items").unwrap();
    let a = Selector::parse("a").unwrap();
    for _div in fragment.select(&div) {
        //println!("{:#?}", _table.inner_html());  
        for _li in _div.select(&li) {
            //println!("{:#?}", _li.inner_html()); 
            let _a = _li.select(&a).next().unwrap();
            let _link = _a.value().attr("href").unwrap_or_default();
            let _title = _a.inner_html();
            _list.push(List{
                timestamp:_today.timestamp(),
                title:_title,
                datetime:"".to_string(),
                link:_link.to_string(),
                images:"".to_string(),
                more:"엠팍".to_string(),
                new: true,
            });
        }
    }
    _list
}

fn load_file_to_list(path:&str) -> Vec<List> {
    let _path = Path::new(&path);
    if _path.exists() {
        let load_list: Vec<List> = serde_json::from_value(utils::file_read_to_json(&path).unwrap_or_default()).unwrap_or_default();
        let _stamp = (Utc::now().with_timezone(&Seoul)).timestamp();
        //172800 48시간 이전 내역 삭제
        load_list.into_iter().map(|mut x|{
            if _stamp - x.timestamp > 28800 {
                x.new = false; 
            }
            x
        }).filter(|x| (_stamp - x.timestamp) < 172800).collect::<Vec<List>>()
    } else {
        vec![]
    }
}

fn newer_to_list(a:Vec<List>, b:Vec<List>) -> Vec<List> {
    let mut _a:Vec<List> = vec![];
    for x in &a {
        _a.push(x.to_owned());
    }
    let mut hash_key = HashSet::new();
    for x in &b {
        hash_key.insert(x.link.to_owned());
    }
    _a = _a.into_iter().filter(|x| hash_key.contains(&x.link) == false).collect::<Vec<List>>();

    _a
}

fn merge_to_list(a:Vec<List>, mut b:Vec<List>) -> Vec<List> {
    let mut h_a = HashSet::new();
    let mut _a:Vec<List> = vec![];
    for x in &a {
        if !h_a.contains(&x.link.to_owned()) {
            _a.push(x.to_owned());
            h_a.insert(x.link.to_owned());
        } 
    }
    let mut hash_key = HashSet::new();
    for x in &b {
        hash_key.insert(x.link.to_owned());
    }
    _a = _a.into_iter().filter(|x| hash_key.contains(&x.link) == false).collect::<Vec<List>>();
    let mut re = vec![];

    re.append(&mut _a);
    re.append(&mut b);
    re.sort_by(|x,y| y.timestamp.cmp(&x.timestamp));
    re
}

