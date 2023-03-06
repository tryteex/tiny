use std::{collections::{HashMap, hash_map::Entry}, sync::{Arc, Mutex}, fs::{read_dir, read_to_string}};

use crate::sys::log::Log;

use super::db::{DBConfig, DB};

#[derive(Debug, Clone)]
pub struct LangItem{
    pub id: u64,
    pub code: String,   //ISO 3166 alpha-2: ua - Ukraine,     us - USA,       gb - United Kingdom
    pub lang: String,   //ISO 639-1       : uk - ukrainian,   en - english,   en - english
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct Lang {
    pub langs: Vec<LangItem>,
    pub avaible: Vec<u64>,
    list: HashMap<u64, HashMap<String, HashMap<String, HashMap<String, String>>>>, // lang_id => module => class => text = translate
}

impl Lang {
    pub fn get<'a>(&self, id: u64, module: &str, class: &str) -> Option<&HashMap<String, String>> {
        if let Some(i) = self.list.get(&id) {
            if let Some(c) = i.get(module) {
                if let Some(v) = c.get(class) {
                    return Some(v);
                };
            };
        }
        None
    }

    pub fn check(&self, lang_id: u64) -> bool {
        self.avaible.contains(&lang_id)
    }

    pub fn new(root: &str, db: &DBConfig, log: Arc<Mutex<Log>>) -> Lang {
        let q = "
            SELECT lang_id, name, lang, code
            FROM lang
            WHERE enable
            ORDER BY sort
        ";
        let res = match DB::one_time_query(db, Arc::clone(&log), q) {
            Some(r) => if r.len() == 0 {
                Log::push_warning(log, 1151, None);
                return Lang {
                    langs: Vec::new(),
                    avaible: Vec::new(),
                    list: HashMap::new(),
                };
            } else {
                r
            },
            None => {
                Log::push_warning(log, 1150, None);
                return Lang {
                    langs: Vec::new(),
                    avaible: Vec::new(),
                    list: HashMap::new(),
                };
            },
        };
        let mut langs = Vec::with_capacity(res.len());
        let mut ids = HashMap::with_capacity(res.len());
        let mut avaible = Vec::new();
        for row in res {
            let id = match u64::try_from(row.get::<usize, i64>(0)) {
                Ok(i) => i,
                Err(_) => {
                    Log::push_warning(log, 1152, None);
                    return Lang {
                        langs: Vec::new(),
                        avaible: Vec::new(),
                        list: HashMap::new(),
                    };
                },
            };
            let code: String = row.get(3);
            ids.insert(code.clone(), id);
            avaible.push(id);
            langs.push(LangItem {
                id,
                code,
                lang: row.get(2),
                name: row.get(1),
            });
        }

        let path = format!("{}/app/", root);
        let mut list: HashMap<u64, HashMap<String, HashMap<String, HashMap<String, String>>>> = HashMap::new();

        match read_dir(path) {
            Ok(r) => {
                for entry in r {
                    if let Ok(e) = entry {
                        let path = e.path();
                        if path.is_dir() {
                            if let Some(m) = path.file_name() {
                                if let Some(module) = m.to_str() {
                                    if let Ok(r) = read_dir(&path) {
                                        for entry in r {
                                            if let Ok(e) = entry {
                                                let path = e.path();
                                                if path.is_dir() {
                                                    if let Some(c) = path.file_name() {
                                                        if let Some(class) = c.to_str() {
                                                            if let Ok(r) = read_dir(&path) {
                                                                for entry in r {
                                                                    if let Ok(e) = entry {
                                                                        let path = e.path();
                                                                        if path.is_file() {
                                                                            if let Some(v) = path.file_name() {
                                                                                if let Some(lang) = v.to_str() {
                                                                                    if lang.ends_with(".lang") && lang.len() > 5 {
                                                                                        if let Some(id) = ids.get(&lang[..lang.len()-5]) {
                                                                                            if let Ok(str) = read_to_string(&path) {
                                                                                                for line in str.lines() {
                                                                                                    let vv: Vec<&str> = line.splitn(2, "=").collect();
                                                                                                    if vv.len() == 2 {
                                                                                                        let key = vv[0].trim().to_owned();
                                                                                                        let val = vv[1].trim_start().to_owned();
                                                                                                        match list.entry(*id) {
                                                                                                            Entry::Occupied(mut o) => match o.get_mut().entry(module.to_owned()) {
                                                                                                                Entry::Occupied(mut o) => match o.get_mut().entry(class.to_owned()) {
                                                                                                                    Entry::Occupied(mut o) => match o.get_mut().entry(key) {
                                                                                                                        Entry::Occupied(mut o) => * o.get_mut() = val,
                                                                                                                        Entry::Vacant(v) => { v.insert(val); },
                                                                                                                    },
                                                                                                                    Entry::Vacant(v) => {
                                                                                                                        let mut kk = HashMap::new();
                                                                                                                        kk.insert(key, val);
                                                                                                                        v.insert(kk);
                                                                                                                    },
                                                                                                                },
                                                                                                                Entry::Vacant(v) => {
                                                                                                                    let mut kk = HashMap::new();
                                                                                                                    kk.insert(key, val);
                                                                                                                    let mut cc = HashMap::new();
                                                                                                                    cc.insert(class.to_owned(), kk);
                                                                                                                    v.insert(cc);
                                                                                                                },
                                                                                                            },
                                                                                                            Entry::Vacant(v) => {
                                                                                                                let mut kk = HashMap::new();
                                                                                                                kk.insert(key, val);
                                                                                                                let mut cc = HashMap::new();
                                                                                                                cc.insert(class.to_owned(), kk);
                                                                                                                let mut mm = HashMap::new();
                                                                                                                mm.insert(module.to_owned(), cc);
                                                                                                                v.insert(mm);
                                                                                                            },
                                                                                                        }
                                                                                                    }
                                                                                                }
                                                                                            }
                                                                                        }
                                                                                    }
                                                                                }
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            },
            Err(e) => Log::push_warning(log, 1153, Some(e.to_string())),
        }
        Lang {
            list,
            avaible,
            langs,
        }
    }

}