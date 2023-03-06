use std::{collections::HashMap, sync::{Mutex, Arc}, io::Write};

use chrono::Local;
use serde::{Serialize, Deserialize};
use tempfile::NamedTempFile;
use sha3::{Digest, Sha3_512};

pub type Act = fn(&mut Action) -> Answer;
pub type ActMap = HashMap<&'static str, HashMap<&'static str, HashMap<&'static str, Act>>>;

use crate::sys::log::Log;

use super::{cache::{Cache}, db::DB, html::{Html, Node}, lang::Lang};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Data {
    None,
    U8(u8),
    I64(i64),
    U64(u64),
    F64(f64),
    Bool(bool),
    String(String),
    Vec(Vec<Data>),
    Map(HashMap<String, Data>),       // Map of string keys
}

enum Route {
    Redirect,
    Ok(String, String, String, Option<String>, Option<u64>),
}

pub struct DataRun<'a> {
    pub cache: Arc<Mutex<Cache>>,
    pub html: &'a Html,
    pub lang: &'a Lang,
    pub salt: &'a str,
    pub lang_id: u64,
    pub path: &'a str,
    pub db: &'a mut DB,
    pub engine: &'a ActMap,
}

#[derive(Debug)]
pub struct Redirect {
    pub url: String,              // Url
    pub permanently: bool,        // Permanently redirect
}

#[derive(Debug)]
pub enum Answer{
    None,               // With out answer
    String(String),     // Answer in the form of text
    Raw(Vec<u8>),       // Answer in binary data
}

#[derive(Debug)]
pub struct WebFile<'a> {
    pub size: usize,                      // File size
    pub name: &'a str,                    // File name
    pub tmp: std::path::PathBuf,      // Absolute path to file location
}

#[derive(Debug)]
pub struct Input<'a> {
    pub get: Option<HashMap<&'a str, &'a str>>,         // GET data
    pub post: Option<HashMap<&'a str, &'a str>>,        // POST data
    pub file: Option<HashMap<&'a str, Vec<WebFile<'a>>>>,  // FILE data
    pub cookie: Option<HashMap<&'a str, &'a str>>,      // Cookies
}

#[derive(Debug)]
pub struct Request<'a> {
    pub ajax: bool,                           // Ajax query (only software detect)
    pub host: &'a str,                       // Request host. Example: subdomain.domain.zone
    pub scheme: &'a str,                     // Request scheme. Example: http / https
    pub agent: &'a str,                          // HTTP_USER_AGENT
    pub referer: &'a str,                        // HTTP_REFERER
    pub ip: &'a str,                             // Client IP
    pub method: &'a str,                         // REQUEST_METHOD
    pub path: &'a str,                           // DOCUMENT_ROOT
    pub dir: &'a str,                            // 
    pub url: &'a str,                            // Request url. Example: /product/view/item/145
    pub input: Input<'a>,
}

#[derive(Debug)]
pub struct Response {
    pub redirect: Option<Redirect>,
    pub http_code: Option<u16>,
    pub css: Vec<String>,                     // Addition css script
    pub js: Vec<String>,                      // Addition js script
}

#[derive(Debug)]
pub struct Session<'a> {
    id: u64,                              // session_id from database
    lang_id: u64,
    pub user_id: u64,                         // user_id from database
    pub role_id: u64,                         // role_id from database
    pub key: &'a str,                           // cookie key
    pub session: String,                      // cookie key
    data: HashMap<String, Data>,     // User data
    change: bool,                         // User data is changed
}

impl<'a> Session<'a> {
    pub fn set_lang(&mut self, lang_id: u64) {
        if self.lang_id != lang_id {
            self.lang_id = lang_id;
            self.change = true;
        }
    }

    pub fn get_lang(&self) -> u64 {
        self.lang_id
    }
}

pub struct Action<'a> {
    pub salt: &'a str,
    pub cache: Arc<Mutex<Cache>>,
    pub db: &'a mut DB,
    pub request: Request<'a>,
    pub response: Response,
    pub session: Session<'a>,
    pub log: Arc<Mutex<Log>>,
    pub module: Option<String>,
    pub class: Option<String>,
    pub action: Option<String>,
    engine: &'a ActMap,
    template: &'a Html,
    pub language: &'a Lang,
    current_module: Option<String>,
    current_class: Option<String>,
    pub html: Option<&'a HashMap<String, Vec<Node>>>,
    lang: Option<&'a HashMap<String, String>>,
    pub internal: bool,
    pub data: HashMap<&'a str, Data>,
    pub param: Option<String>,
}

impl<'a> Action<'a> {
    pub fn new(param: &'a HashMap<String, String>, stdin: &'a Option<Vec<u8>>, data: DataRun<'a>, log: Arc<Mutex<Log>>) -> Action<'a> {
        let ajax = match param.get("HTTP_X_REQUESTED_WITH") {
            Some(a) => a.to_lowercase().eq("xmlhttprequest"),
            None => false,
        };
        let host = match param.get("HTTP_HOST") {
            Some(h) => h,
            None => "",
        };
        let scheme = match param.get("REQUEST_SCHEME") {
            Some(s) => s,
            None => "https",
        };
        let agent = match param.get("HTTP_USER_AGENT") {
            Some(a) => a,
            None => "",
        };
        let referer = match param.get("HTTP_REFERER") {
            Some(r) => r,
            None => "",
        };
        let ip = match param.get("REMOTE_ADDR") {
            Some(i) => i,
            None => "",
        };
        let method = match param.get("REQUEST_METHOD") {
            Some(m) => m,
            None => "",
        };
        let path = match param.get("DOCUMENT_ROOT") {
            Some(a) => a,
            None => data.path,
        };

        let url = match param.get("REDIRECT_URL") {
            Some(u) => match u.splitn(2, '?').next() {
                Some(s) => s,
                None => "",
            },
            None => "",
        };
        // Extract GET data 
        let get = match param.get("QUERY_STRING") {
            Some(v) => {
                if v.len() > 0 {
                    let gets:Vec<&str> = v.split("&").collect();
                    let mut list = HashMap::with_capacity(gets.len());
                    for v in gets {
                        let key: Vec<&str> = v.splitn(2, "=").collect();
                        match key.len() {
                            1 => list.insert(v, ""),
                            _ => list.insert(key[0], key[1]),
                        };
                    }
                    if list.len() == 0 {
                        None
                    } else {
                        Some(list)
                    }
                } else {
                    None
                }
            },
            None => None,
        };

        let post;
        let file;
        // Extract POST data 
        match param.get("CONTENT_TYPE") {
            Some(c) => {
                if c == "application/x-www-form-urlencoded" {
                    //Simple post
                    post = match &stdin {
                        Some(d) => match std::str::from_utf8(d) {
                            Ok(s) => {
                                if s.len() == 0 {
                                    None
                                } else {
                                    let val: Vec<&str> = s.split("&").collect();
                                    let mut list = HashMap::with_capacity(val.len());
                                    for v in val {
                                        let val: Vec<&str> = v.splitn(2, "=").collect();
                                        match val.len() {
                                        1 => list.insert(v, ""),
                                        _ => list.insert(val[0], val[1]),
                                        };
                                    }
                                    if list.len() == 0 {
                                        None
                                    } else {
                                        Some(list)
                                    }
                                }
                            },
                            Err(_) => None,
                        }
                        None => None,
                    };
                    file = None;
                } else if c.len() > 30 {
                    if let "multipart/form-data; boundary=" = &c[..30] { 
                        // Multi post with files
                        let boundary = format!("--{}", &c[30..]);
                        let stop: [u8; 4] = [13, 10, 13, 10];
                        match &stdin {
                            Some(data) => {
                                if data.len() == 0 {
                                    post = None;
                                    file = None;
                                } else {
                                    let mut seek: usize = 0;
                                    let mut start: usize;
                                    let b_len = boundary.len();
                                    let len = data.len() - 4;
                                    let mut found: Option<(usize, &str)> = None;
                                    let mut list_post = HashMap::new();
                                    let mut list_file = HashMap::new();
                                    while seek < len {
                                        // Find a boundary
                                        if boundary.as_bytes() == &data[seek..seek + b_len] {
                                            if seek + b_len == len {
                                                if let Some((l, h)) = found {
                                                    let d = &data[l..seek - 2];
                                                    Action::get_post_file(h, d, &mut list_post, &mut list_file);
                                                };
                                                break;
                                            }
                                            seek += b_len + 2;
                                            start = seek;
                                            while seek < len {
                                                if stop == data[seek..seek+4] {
                                                    if let Ok(s) = std::str::from_utf8(&data[start..seek]) {
                                                        if let Some((l, h)) = found {
                                                            let d = &data[l..start-b_len-4];
                                                            Action::get_post_file(h, d, &mut list_post, &mut list_file);
                                                        };
                                                        found = Some((seek+4, s));
                                                    }
                                                    seek += 4;
                                                    break;
                                                } else {
                                                    seek += 1;
                                                }
                                            }
                                        } else {
                                            seek += 1;
                                        }
                                    }
                                    if list_post.len() == 0 {
                                        post = None;
                                    } else {
                                        post = Some(list_post);
                                    }
                                    if list_file.len() == 0 {
                                        file = None;
                                    } else {
                                        file = Some(list_file);
                                    }
                                }
                            },
                            None => {
                                post = None;
                                file = None;
                            },
                        };
                    } else {
                        post = None;
                        file = None;
                    }
                } else {
                    post = None;
                    file = None;
                }
            },
            None => {
                post = None;
                file = None;
            },
        }

        let session_key;
        let tiny_key = "tinysession";
        // Extract COOKIE data 
        let cookie = match param.get("HTTP_COOKIE") {
            Some(c) => {
                if c.len() > 0 {
                    let cooks:Vec<&str> = c.split("; ").collect();
                    let mut list = HashMap::with_capacity(cooks.len());
                    let mut ses = None;
                    'cook: for v in cooks {
                        let key: Vec<&str> = v.splitn(2, "=").collect();
                        if key.len() == 2 {
                            if key[0] == tiny_key {
                                if key[1].len() == 128 {
                                    for b in key[1].as_bytes() {
                                        if !((*b > 47 && *b < 58) || (*b > 96 && *b < 103)) {
                                            continue 'cook;
                                        }
                                    }
                                    ses = Some(key[1]);
                                }
                            } else {
                                list.insert(key[0], key[1]);
                            }
                        }
                    }
                    match ses {
                        Some(s) => session_key = s.to_owned(),
                        None => session_key = Action::generate_session(&data.salt, ip, agent, host),
                    };
                    if list.len() == 0 {
                        None
                    } else {
                        Some(list)
                    }
                } else {
                    session_key = Action::generate_session(&data.salt, ip, agent, host);
                    None
                }
            },
            None => {
                session_key = Action::generate_session(&data.salt, ip, agent, host);
                None
            },
        };
        let session_id;
        let session_user_id;
        let session_role_id;
        let session_data;
        let session_lang_id;
        let session_change;

        if let Some((sid, uid, rid, lid, sdata)) = Action::load_session(&session_key, ip, agent, data.db, data.lang_id) {
            session_id = sid;
            session_user_id = uid;
            session_role_id = rid;
            session_data = sdata;
            session_lang_id = lid;
            session_change = false;
        } else {
            session_lang_id = data.lang_id;
            session_id = 0;
            session_user_id = 0;
            session_role_id = 0;
            session_data = HashMap::new();
            session_change = true;
        }

        let session = Session {
            id: session_id,
            lang_id: session_lang_id,
            user_id: session_user_id,
            role_id: session_role_id,
            key: tiny_key,
            session: session_key,
            data: session_data,
            change: session_change,
        };

        let input = Input {
            get,
            post,
            file,
            cookie,
        };

        let request = Request {
            ajax,
            host,
            scheme,
            agent,
            referer,
            ip,
            method,
            path,
            dir: data.path,
            url,
            input,
        };

        let response = Response {
            redirect: None,
            http_code: None,
            css: Vec::with_capacity(2),
            js: Vec::with_capacity(2),
        };

        Action {
            salt: &data.salt,
            cache: data.cache,
            db: data.db,
            request,
            response,
            session,
            log,
            module: None,
            class: None,
            action: None,
            engine: data.engine,
            current_module: None,
            current_class: None,
            template: data.html,
            language: data.lang,
            html: None,
            lang: None,
            internal: false,
            data: HashMap::with_capacity(256),
            param: None,
        }
    }

    pub fn lang(&self, text: &str) -> String {
        if let Some(l) = self.lang {
            if let Some(str) = l.get(text) {
                return str.to_owned();
            }
        }
        text.to_owned()
    }

    pub fn stop(mut self) {
        self.save_session();
    }

    fn load_session(key: &str, ip: &str, agent: &str, db: &'a mut DB, lang_id: u64) -> Option<(u64, u64, u64, u64, HashMap<String, Data>)> {
        let res = match db.query_fast(0, &[&key, &ip, &agent, &(lang_id as i64), &key]) {
            Some(r) => r,
            None => return None,
        };
        if res.len() == 0 {
            return None;
        }
        let row = &res[0];
        let session_id: i64 = row.get(0);
        let user_id: i64 = row.get(1);
        let role_id: i64 = row.get(2);
        let data: &[u8] = row.get(3);
        let lang_id: i64 = row.get(4);

        let res = if data.len() == 0 {
            HashMap::new()
        } else {
            match bincode::deserialize::<HashMap<String, Data>>(data) {
                Ok(r) => r,
                Err(_) => HashMap::new(),
            }
        };

        Some((session_id as u64, user_id as u64, role_id as u64, lang_id as u64, res))
    }

    fn save_session(&mut self) {
        if self.db.is_not_empty() && self.session.id > 0 {
            if self.session.change {
                let data = match bincode::serialize(&self.session.data) {
                    Ok(r) => r,
                    Err(_) => Vec::new(),
                };
                self.db.query_fast(1, &[&(self.session.user_id as i64), &data, &self.request.ip, &self.request.agent, &(self.session.lang_id as i64), &(self.session.id as i64)]);
            } else {
                self.db.query_fast(2, &[&(self.session.id as i64)]);
            }
        }
    }
    
    fn generate_session(salt: &str, ip: &str, agent: &str, host: &str) -> String {
        // Generate a new cookie
        let time = Local::now().format("%Y.%m.%d %H:%M:%S%.9f %:z").to_string();
        let cook = format!("{}{}{}{}{}", salt, ip, agent, host, time);
        let mut hasher = Sha3_512::new();
        hasher.update(cook.as_bytes());
        format!("{:#x}", hasher.finalize())
    }

    pub fn get_access(&mut self, module: &str, class: &str, action: &str) -> bool {
        let key = format!("auth:{}:{}:{}:{}", self.session.role_id, module, class, action);
        if let Some(data) = Cache::get(Arc::clone(&self.cache), &key, Arc::clone(&self.log)) {
            if let Data::Bool(a) = data {
                return a;
            }
        };
        // Prepare sql query
        match self.db.query_fast(3, &[&(self.session.user_id as i64), &module, &module, &module, &class, &class, &action]) {
            Some(rows) => {
                if rows.len() == 1 {
                    let access: bool = rows[0].get(0);
                    Cache::set(Arc::clone(&self.cache), key, Data::Bool(access), Arc::clone(&self.log));
                    access
                } else {
                    Cache::set(Arc::clone(&self.cache), key, Data::Bool(false), Arc::clone(&self.log));
                    false
                }
            },
            None => false,
        }
    }

    pub fn run(&mut self) -> Answer {
        self.db.check();

        let (module, class, action, param, lang_id) = match self.extract_route() {
            Route::Redirect => return Answer::None,
            Route::Ok(m, c, a, p, l) => (m, c, a, p, l),
        };
        self.module = Some(module.clone());
        self.class = Some(class.clone());
        self.action = Some(action.clone());
        if let Some(lang_id) = lang_id {
            if self.session.lang_id != lang_id {
                self.session.change = true;
                self.session.lang_id = lang_id;
            }
        }
        self.start_route(&module, &class, &action, param, false)
    }

    pub fn not_found(&self) -> String {
        if let Some(data) = Cache::get(Arc::clone(&self.cache), &format!("404:{}", self.session.lang_id), Arc::clone(&self.log)) {
            if let Data::String(url) = data {
                return url;
            }
        } else if let Some(data) = Cache::get(Arc::clone(&self.cache), "404", Arc::clone(&self.log)) {
            if let Data::String(url) = data {
                return url;
            }
        };
        "/index/index/not_found".to_owned()
    }

    fn start_route(&mut self, module: &str, class: &str, action: &str, param: Option<String>, internal: bool) -> Answer {
        if self.get_access(module, class, action) {
            return self.invoke(module, class, action, param, internal);
        }
        if internal {
            return Answer::None;
        }
        if !(module == "index" && class == "index" && action == "not_found") {
            self.response.redirect = Some(Redirect { url: self.not_found(), permanently: false});
        }
         Answer::None
    }

    fn compare(&self, module: &str, class: &str) -> bool {
        match &self.current_module {
            Some(m) => if m != module {
                return false;
            },
            None => return false,
        };
        match &self.current_class {
            Some(c) => if c != class {
                return false;
            },
            None => return false,
        };
        true
    }

    fn invoke(&mut self, module: &str, class: &str, action: &str, param: Option<String>, internal: bool) -> Answer {
        if let Some(m) = &self.engine.get(module) {
            if let Some(c) = m.get(class) {
                if let Some(a) = c.get(action) {
                    if self.compare(module, class) {
                        let i = self.internal;
                        let p = match param {
                            Some(str) => self.param.replace(str),
                            None => self.param.take(),
                        };
                        self.internal = internal;
                        let res = a(self);
                        self.internal = i;
                        self.param = p;
                        return res;
                    } else {
                        let h = self.html;
                        let l = self.lang;
                        let i = self.internal;
                        let p = match param {
                            Some(str) => self.param.replace(str),
                            None => self.param.take(),
                        };
                        let m = self.current_module.replace(module.to_owned());
                        let c = self.current_class.replace(class.to_owned());
                        self.html = self.template.get(module, class);
                        self.lang = self.language.get(self.session.lang_id, module, class);
                        self.internal = internal;
                        let res = a(self);
                        self.current_module = m;
                        self.current_class = c;
                        self.html = h;
                        self.lang = l;
                        self.internal = i;
                        self.param = p;
                        return res;
                    }
                }
            }
        }
        Answer::None
    }

    // Load internal controller
    pub fn load_raw(&mut self, module: &str, class: &str, action: &str, param: Option<String>) -> Answer {
        self.start_route(module, class, action, param, true)
    }

    // Load internal controller and set value
    pub fn load(&mut self, key: &'a str, module: &str, class: &str, action: &str, param: Option<String>) {
        if let Answer::String(str) = self.start_route(module, class, action, param, true) {
            self.data.insert(key, Data::String(str));
        }
    }

    fn extract_route(&mut self) -> Route {

        // Get redirect
        let key = format!("redirect:{}", &self.request.url);
        if let Some(data) = Cache::get(Arc::clone(&self.cache), &key, Arc::clone(&self.log)) {
            if let Data::String(r) = data {
                let permanently = if &r[..1] == "1" { true } else { false };
                self.response.redirect = Some(Redirect { url: r[1..].to_owned(), permanently});
                return Route::Redirect;
            }
        }
    
        // Get route
        let key = format!("route:{}", &self.request.url);
        if let Some(data) = Cache::get(Arc::clone(&self.cache), &key, Arc::clone(&self.log)) {
            if let Data::Vec(r) = data {
                if let Data::String(module) = &r[0] {
                    if let Data::String(class) = &r[1] {
                        if let Data::String(action) = &r[2] {
                            let lang = match &r[4] {
                                Data::U64(lang_id) => Some(*lang_id),
                                _ => None,
                            };
                            let param = match &r[3] {
                                Data::String(param) => Some(param.clone()),
                                _ => None,
                            };
                            return Route::Ok(module.to_owned(), class.to_owned(), action.to_owned(), param, lang);
                        }
                    }
                }
            }
        }

        let module;
        let class;
        let action;
        let param;
        if self.request.url != "/" {
            let load: Vec<&str> = self.request.url.splitn(5, "/").collect();
            match load.len() {
                2 => {
                    module = load[1];
                    class = "index";
                    action = "index";
                    param = None;
                },
                3 => {
                    module = load[1];
                    class = load[2];
                    action = "index";
                    param = None;
                },
                4 => {
                    module = load[1];
                    class = load[2];
                    action = load[3];
                    param = None;
                },
                5 => {
                    module = load[1];
                    class = load[2];
                    action = load[3];
                    param = Some(load[4].to_owned());
                },
                _ => {
                    module = "index";
                    class = "index";
                    action = "index";
                    param = None;
                }
            } 
        } else {
            module = "index";
            class = "index";
            action = "index";
            param = None;
        }
        Route::Ok(module.to_owned(), class.to_owned(), action.to_owned(), param, None)
    }

    // get post file from multipart/form-data
    fn get_post_file(header: &'a str, data: &'a [u8], post: &mut HashMap<&'a str, &'a str>, file: &mut HashMap<&'a str, Vec<WebFile<'a>>>) {
        let h: Vec<&str> = header.splitn(3, "; ").collect();
        let len = h.len();
        if len == 2 {
            if let Ok(v) = std::str::from_utf8(data) {
                let k = &h[1][6..h[1].len() - 1];
                post.insert(k, v);
            }
        } else if len == 3 {
            let k = &h[1][6..h[1].len() - 1];
            let n: Vec<&str> = h[2].splitn(2, "\r\n").collect();
            let n = &n[0][10..n[0].len()-1];

            if let Ok(tmp) = NamedTempFile::new() {
                if let Ok((mut f, p)) = tmp.keep() {
                    if let Ok(_) = f.write_all(data) {
                        if let None = file.get(&k) {
                            file.insert(k, Vec::with_capacity(16));
                        }
                        if let Some(d) = file.get_mut(&k) {
                            d.push(WebFile { size: data.len(), name: n, tmp: p})
                        };
                    }
                }
            }
        }
    }

    pub fn http_code_get(code: u16) -> String {
        let mut s = String::with_capacity(48);
        s.push_str(&code.to_string());
        match code {
          100 => s.push_str(" Continue"),
          101 => s.push_str(" Switching Protocols"),
          102 => s.push_str(" Processing"),
          103 => s.push_str(" Early Hints"),
          200 => s.push_str(" OK"),
          201 => s.push_str(" Created"),
          202 => s.push_str(" Accepted"),
          203 => s.push_str(" Non-Authoritative Information"),
          204 => s.push_str(" No Content"),
          205 => s.push_str(" Reset Content"),
          206 => s.push_str(" Partial Content"),
          207 => s.push_str(" Multi-Status"),
          208 => s.push_str(" Already Reported"),
          226 => s.push_str(" IM Used"),
          300 => s.push_str(" Multiple Choices"),
          301 => s.push_str(" Moved Permanently"),
          302 => s.push_str(" Found"),
          303 => s.push_str(" See Other"),
          304 => s.push_str(" Not Modified"),
          305 => s.push_str(" Use Proxy"),
          306 => s.push_str(" (Unused)"),
          307 => s.push_str(" Temporary Redirect"),
          308 => s.push_str(" Permanent Redirect"),
          400 => s.push_str(" Bad Request"),
          401 => s.push_str(" Unauthorized"),
          402 => s.push_str(" Payment Required"),
          403 => s.push_str(" Forbidden"),
          404 => s.push_str(" Not Found"),
          405 => s.push_str(" Method Not Allowed"),
          406 => s.push_str(" Not Acceptable"),
          407 => s.push_str(" Proxy Authentication Required"),
          408 => s.push_str(" Request Timeout"),
          409 => s.push_str(" Conflict"),
          410 => s.push_str(" Gone"),
          411 => s.push_str(" Length Required"),
          412 => s.push_str(" Precondition Failed"),
          413 => s.push_str(" Content Too Large"),
          414 => s.push_str(" URI Too Long"),
          415 => s.push_str(" Unsupported Media Type"),
          416 => s.push_str(" Range Not Satisfiable"),
          417 => s.push_str(" Expectation Failed"),
          418 => s.push_str(" (Unused)"),
          421 => s.push_str(" Misdirected Request"),
          422 => s.push_str(" Unprocessable Content"),
          423 => s.push_str(" Locked"),
          424 => s.push_str(" Failed Dependency"),
          425 => s.push_str(" Too Early"),
          426 => s.push_str(" Upgrade Required"),
          428 => s.push_str(" Precondition Required"),
          429 => s.push_str(" Too Many Requests"),
          431 => s.push_str(" Request Header Fields Too Large"),
          451 => s.push_str(" Unavailable For Legal Reasons"),
          500 => s.push_str(" Internal Server Error"),
          501 => s.push_str(" Not Implemented"),
          502 => s.push_str(" Bad Gateway"),
          503 => s.push_str(" Service Unavailable"),
          504 => s.push_str(" Gateway Timeout"),
          505 => s.push_str(" HTTP Version Not Supported"),
          506 => s.push_str(" Variant Also Negotiates"),
          507 => s.push_str(" Insufficient Storage"),
          508 => s.push_str(" Loop Detected"),
          510 => s.push_str(" Not Extended (OBSOLETED)"),
          511 => s.push_str(" Network Authentication Required"),
          _ => s.push_str(" Unassigned"),
        };
        s
      }
}