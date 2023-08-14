#[macro_use]
extern crate serde_derive;

use std::path::Path;
use std::collections::HashSet;
use scraper::{Html, Selector};
use chrono::{Duration, Utc,};
use reqwest::header;
use anyhow::Result;
use std::io::Write;

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
pub struct Command {
    pub exe: String,
    pub args: Vec<String>,
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
    pub basepath: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Images {
    pub link: String,
    pub title: String,
    pub filename: String,
    pub path: String,
    pub ext: String,
}

const SAVE_PATH: &str = "./save.json";
const SITE_PATH: &str = "./site.json";
const DOWN_PATH: &str = "./down.json";
const DOWN_PATH_ROOT: &str = r"P:\Comic";

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let mut site_list : Vec<Site> = serde_json::from_value(utils::file_read_to_json(SITE_PATH).unwrap_or_default()).unwrap_or_default();
    let mut save_list : Vec<Save> = serde_json::from_value(utils::file_read_to_json(SAVE_PATH).unwrap_or_default()).unwrap_or_default();
    let mut down_list : Vec<Down> = serde_json::from_value(utils::file_read_to_json(DOWN_PATH).unwrap_or_default()).unwrap_or_default();
    

    let __loop = tokio::task::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(900));
        loop {
            interval.tick().await;
        
            let mut dc_list: Vec<List>  = vec![];
            let mut dc_down_list: Vec<List>  = vec![];
            let mut fm_list: Vec<List>  = vec![];
            let mut mp_list: Vec<List>  = vec![];
            
            let mut down_image_list = vec![];

            for _site in site_list.iter_mut() {
                match _site.host.as_ref() {
                    "dc" => {
                        let mut _dc_list: Vec<List> = get_dc(&_site.url).await.unwrap_or_default();
                        dc_down_list = _dc_list.clone();
                        dc_list.append(&mut _dc_list);
                    },
                    "fm" => {
                        let mut _fm_list: Vec<List> = get_fm(&_site.url).await.unwrap_or_default();
                        fm_list.append(&mut _fm_list);
                    },
                    "mp" => {
                        let mut _mp_list: Vec<List> = get_mp(&*_site.url).await.unwrap_or_default();
                        mp_list.append(&mut _mp_list);
                    },
                    "mp_low" => {
                        let mut _mp_list: Vec<List> = get_mp_part_low(&*_site.url).await.unwrap_or_default();
                        mp_list.append(&mut _mp_list);
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
                //println!("{:#?}", &_downlink.title);  
                for _downtarget in down_list.iter_mut() {
                    let title = _downlink.title.to_string();
                    //println!("{:#?}", &title); 
                    //println!("{:#?}", &_downtarget.title); 
                    if title.contains(&_downtarget.title) {
                        //println!("{:#?}", &_downlink.title);    
                        let mut _image_list: Vec<Images> = get_dcimage(&*_downlink.link,&*_downlink.title,&*_downtarget.title).await.unwrap_or_default();
                        down_image_list.append(&mut _image_list);
                    }
                }
            }

            for _down in down_image_list.iter_mut() {
                //println!("{:#?}", &_down.link); 
                let mut _loadfile = download_image_to(&*_down.link, &*_down.path,&*_down.filename,&*_down.title).await.unwrap_or_default();
                //println!("{:#?}", _loadfile);    
            }
            println!("End Of job");    
            //git_push().await.unwrap_or_default();
        }
    });
    __loop.await?
}



fn load_file_to_list(path:&str) -> Vec<List> {
    let _path = Path::new(&path);
    if _path.exists() {
        let load_list: Vec<List> = serde_json::from_value(utils::file_read_to_json(&path).unwrap_or_default()).unwrap_or_default();
        let _stamp = (Utc::now() + Duration::hours(9)).timestamp();
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

async fn get_dcimage(link: &str,title: &str,part_title: &str) ->Result<Vec<Images>, reqwest::Error> {
    let mut nums = 1;
    let mut _list: Vec<Images> = vec![];
    //println!("{:#?}", link);    
    let _url = format!("{}{}","https://gall.dcinside.com",link);

    let resp = reqwest::get(_url).await.unwrap().text().await.unwrap();

    //println!("{:#?}", resp);    

    let fragment = Html::parse_fragment(&resp);
    let images = Selector::parse(r#"img"#).unwrap();

    for element in fragment.select(&images) {
        //println!("{:#?}", element.value());    
        let url = element.value().attr("src").unwrap_or_default();
        //let ev = element.value().attr("onmouseover").unwrap_or_default();
        if url.contains("/viewimage.php?no=") {
            //println!("{:#?}", &url);    
            _list.push(Images{
                link: url.to_string(),
                filename:format!("{}{}",nums,".jpg"),
                title:part_title.to_string(),
                path:format!("{}",title),
                ext:"jpg".to_string(),
            });
            nums += 1;
        }
    }
    Ok(_list)
}

async fn download_image_to(url: &str, path_name: &str, file_name: &str, part_title: &str) ->Result<String> {
    // Send an HTTP GET request to the URL
    let response = reqwest::get(url).await.unwrap().bytes().await.unwrap();
    // Create a new file to write the downloaded image to
    let frag = scraper::Html::parse_fragment(path_name);
    let mut _title: String = "".to_string();
    for node in frag.tree {
        if let scraper::node::Node::Text(text) = node {
            //print!("{}", text.text);
            _title = text.text.to_string();
        }
    }

    let parts = _title.split(&part_title).next().unwrap();
    let _path = format!(r"{}\{}{}",&DOWN_PATH_ROOT,&part_title,&parts);
    let path = _path.trim();
    if !utils::folder_exist(&path) {
        //println!("{:#?}", &path);
        std::fs::create_dir_all(&path)?;
    }
    //println!("{:#?}", format!(r"{}\{}",&path,&file_name));

    let mut file : std::fs::File = std::fs::File::create(format!(r"{}\{}",&path,&file_name)).unwrap();
    file.write_all(&response).unwrap();

    Ok("ok".to_string())
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

async fn get_dc(_url : &str) -> Result<Vec<List>, reqwest::Error>  {
    let mut _list: Vec<List> = vec![];
    let _today = Utc::now() + Duration::hours(9);

    println!("{:#?}", _url);    
    let resp =  reqwest::get(_url)
        .await?
        .text()
        .await?;

    let fragment = Html::parse_fragment(&resp);
    let part = Selector::parse("tr.ub-content").unwrap();
    let title = Selector::parse("td.gall_tit > a").unwrap();
    let date = Selector::parse("td.gall_date").unwrap();
    
    for element in fragment.select(&part) {
        let td1 = element.select(&title).next().unwrap();
        let _title = td1.inner_html();

        let _link = td1.value().attr("href").unwrap_or_default();
        let _date = element.select(&date).next().unwrap().value().attr("title").unwrap_or_default();
        let _date_text = element.select(&date).next().unwrap().inner_html();
        let _timestamp = chrono::NaiveDateTime::parse_from_str(_date,"%Y-%m-%d %H:%M:%S");
        match _timestamp {
            Ok(v) => {
                //게시물 시간
                let _diff = _today.timestamp() - v.timestamp();
                //println!("{:#?}, {:#?}, {:#?}", v.timestamp(), _diff, _title);
                if _diff < 172800 {
                    //println!("{:#?}, {:#?}, {:#?}", _title, _link, _date_text);
                    _list.push(List{
                        timestamp: _today.timestamp(),
                        title:_title,
                        datetime:_date_text,
                        link:_link.to_string(),
                        images:"".to_string(),
                        more:"디시".to_string(),
                        new: true,
                    });
                }
            },
            Err(_) =>{
                //
                utils::logger(_link);
            },
        }
    }
    Ok(_list)
}

async fn get_fm(_url : &str) -> Result<Vec<List>, reqwest::Error>  {
    let mut _list: Vec<List> = vec![];
    let _today = Utc::now() + Duration::hours(9);
    let client = reqwest::Client::builder().build().unwrap();

    println!("{:#?}", _url);    
    let resp = client.get(_url)
        .send()
        .await?
        .text()
        .await?;

    let fragment = Html::parse_fragment(&resp);
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
    Ok(_list)
}

async fn get_mp(_url : &str) -> Result<Vec<List>, reqwest::Error>  {

    static APP_USER_AGENT: &str = concat!(
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:98.0) Gecko/20100101 Firefox/98.0"
    );
    let mut headers = header::HeaderMap::new();
    headers.insert(header::AUTHORIZATION, header::HeaderValue::from_static("secret"));
    let client = reqwest::Client::builder()
    .user_agent(APP_USER_AGENT)
    .default_headers(headers)
    .build()?;
    
    let mut _list: Vec<List> = vec![];

    println!("{:#?}", &_url);  
    let resp = client.get(&*_url)
    .send()
    .await?
    .text()
    .await?;

    let fragment = Html::parse_fragment(&resp);
    let _today = Utc::now() + Duration::hours(9);
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
    Ok(_list)
}

async fn get_mp_part_low(_url : &str) -> Result<Vec<List>, reqwest::Error>  {
    static APP_USER_AGENT: &str = concat!(
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:98.0) Gecko/20100101 Firefox/98.0"
    );
    let mut headers = header::HeaderMap::new();
    headers.insert(header::AUTHORIZATION, header::HeaderValue::from_static("secret"));
    let client = reqwest::Client::builder()
    .user_agent(APP_USER_AGENT)
    .default_headers(headers)
    .build()?;
    
    let mut _list: Vec<List> = vec![];
    let _today = Utc::now() + Duration::hours(9);

    println!("{:#?}", &_url);  
    let resp = client.get(&*_url)
    .send()
    .await?
    .text()
    .await?;

    let fragment = Html::parse_fragment(&resp);

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
    Ok(_list)
}


