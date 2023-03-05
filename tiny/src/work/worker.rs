use std::{thread::{self, JoinHandle}, net::TcpStream, sync::{Arc, Mutex, mpsc::{Receiver, Sender}, RwLock}, any::Any, collections::HashMap, fs::remove_file};

use chrono::{Utc, Duration};

use crate::sys::{log::Log, fastcgi::FastCGI};

use super::{cache::Cache, action::{DataRun, Action, Answer, ActMap}, db::{DB, DBConfig}, html::Html, lang::Lang};

pub enum MessageWork {
    Terminate,
    Job(TcpStream),
}

const ON_YEAR: i64 = 31622400;

pub struct Worker {
    thread: JoinHandle<()>,
}

impl Worker {
    pub fn new(
        id: u8, 
        receiver: Receiver<MessageWork>, 
        sender_ready: Arc<Mutex<Sender<u8>>>, 
        cache: Arc<Mutex<Cache>>, 
        html: Arc<RwLock<Html>>, 
        lang: Arc<RwLock<Lang>>, 
        log: Arc<Mutex<Log>>, 
        salt: String, 
        path: String, 
        db: DBConfig, 
        timezone: String, 
        lang_id: u64, 
        engine: ActMap
    ) -> Worker {
        let tlog = Arc::clone(&log);
        let thread = thread::spawn(move || {
            let mut db = DB::new(db.clone(), Arc::clone(&log), timezone.clone(), Arc::clone(&cache));
            let html = match RwLock::read(&html) {
                Ok(h) => h,
                Err(e) => Log::error(log, e.to_string()),
            };
            let lang = match RwLock::read(&lang) {
                Ok(h) => h,
                Err(e) => Log::error(log, e.to_string()),
            };
            loop {
                match Mutex::lock(&sender_ready) {
                    Ok(s) => if let Err(e) = s.send(id) {
                        Log::push_error(log, 703, Some(e.to_string()));
                    },
                    Err(e) => Log::error(log, e.to_string()),
                };
                match receiver.recv() {
                    Ok(e) => match e {
                        MessageWork::Terminate => break,
                        MessageWork::Job(tcp) => {
                            let data = DataRun {
                                cache: Arc::clone(&cache),
                                html: &html,
                                lang: &lang,
                                salt: &salt,
                                lang_id,
                                path: &path,
                                db: &mut db,
                                engine: &engine,
                            };
                            FastCGI::run(&Worker::run, tcp, data, Arc::clone(&log))
                        },
                    },
                    Err(e) => Log::push_error(tlog, 700, Some(e.to_string())),
                };
            }
        });

        Worker {
            thread,
        }
    }

    pub fn join(self) -> Result<(), Box<dyn Any + Send>> {
        self.thread.join()
    }

    fn run(param: HashMap<String, String>, stdin: Option<Vec<u8>>, data: DataRun, log: Arc<Mutex<Log>>) -> Vec<u8> {
        let mut action = Action::new(&param, &stdin, data, Arc::clone(&log));
        let mut result = match action.run() {
            Answer::Raw(answer) => answer,
            Answer::String(answer) => answer.into_bytes(),
            Answer::None => Vec::new(),
        };
        let mut answer: Vec<String> = Vec::with_capacity(16);
        answer.push("HTTP/1.1 ".to_owned());
        if let Some(redirect) = action.response.redirect.as_ref() {
            if redirect.permanently {
                answer.push(format!("{}\r\n", Action::http_code_get(301)));
            } else {
                answer.push(format!("{}\r\n", Action::http_code_get(302)));
            }
            answer.push(format!("Location: {}\r\n", redirect.url));
        } else if let Some(code) = action.response.http_code {
            answer.push(format!("{}\r\n", Action::http_code_get(code)));
        } else {
            answer.push(format!("{}\r\n", Action::http_code_get(200)));
        }
        let time = Utc::now() + Duration::seconds(ON_YEAR);
        let date: String = time.format("%a, %d-%b-%Y %H:%M:%S GMT").to_string();
        let secure = if action.request.scheme == "https" {
            "Secure; "
        } else {
            ""
        };
        answer.push(format!("Set-Cookie: {}={}; Expires={}; Max-Age={}; path=/; domain={}; {}SameSite=none\r\n", action.session.key, action.session.session, date, ON_YEAR, action.request.host, secure));
        answer.push("Connection: keep-alive\r\n".to_owned());
        answer.push("Content-Type: text/html; charset=utf-8\r\n".to_owned());
        answer.push(format!("Content-Length: {}\r\n", result.len()));
        answer.push("\r\n".to_owned());
        let mut answer = answer.join("").into_bytes();
        answer.append(&mut result);

        if let Some(list) = &action.request.input.file {
            for (_, val) in list {
                for f in val {
                    if let Err(e) = remove_file(&f.tmp) {
                        Log::push_warning(Arc::clone(&log), 1020, Some(format!("filename={}. Error={}", &f.tmp.display(), e.to_string())));
                    };
                }
            }
        }
        action.stop(); 
        answer
    }
}