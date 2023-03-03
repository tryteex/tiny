use std::{sync::{Arc, Mutex}, env,fs::{read_to_string, OpenOptions}, io::ErrorKind, net::IpAddr, str::FromStr};
use serde_json::Value;

use crate::work::db::DBConfig;

use super::log::Log;

#[derive(Debug, Clone)]
pub struct Config {
    pub version: String,
    pub max: u8,
    pub bind_accept: IpAddr,
    pub bind_port: u16,
    pub bind_ip: IpAddr,
    pub rpc_port: u16,
    pub rpc_ip: IpAddr,
    pub rpc_accept: IpAddr,
    pub zone: String,
    pub salt: String,
    pub lang_id: u8,
    pub db: DBConfig,
}

#[derive(Debug, Clone, Copy)]
pub enum Mode {
    Start,
    Stop,
    Help,
    Go,
}


#[derive(Debug)]
pub struct Init {
    pub mode: Mode,
    pub conf: Config,
    pub exe_file: String,
    pub exe_path: String,
    pub conf_file: String,
    pub root_path: String,
}

impl Init {
    pub fn new(log: Arc<Mutex<Log>>) -> Option<Init> {
        let exe_file = match env::current_exe() {
            Ok(e) => match e.to_str() {
                Some(e) => if &e[..2] == "\\\\" {
                    if &e[..4] == "\\\\?\\" {
                        e[4..].replace("\\", "/")
                    } else {
                        Log::push_stop(log, 12, Some(e.to_string()));
                        return None;
                    }
                } else {
                    e.replace("\\", "/")
                },
                None => {
                    Log::push_stop(log, 11, Some(e.to_string_lossy().to_string()));
                    return None;
                },
            },
            Err(e) => {
                Log::push_stop(log, 10, Some(e.to_string()));
                return None;
            },
        };
        let mut mode = Mode::Help;
        let mut conf_found = false;
        let mut conf = None;
        let mut args = env::args();
        let mut conf_file = "".to_owned();
        args.next();
        match args.next() {
            None => conf_found = true,
            Some(arg) => match arg.as_str() {
                "-r" => {
                    conf_found = true;
                    match args.next() {
                        Some(p) => conf = {
                            let file = format!("{}/tiny.conf", p);
                            match read_to_string(&file) {
                                Ok(s) => {
                                    conf_file = file;
                                    Some(s)
                                },
                                Err(e) => {
                                    Log::push_stop(log, 14, Some(format!("{}. Error: {}", &p, e.to_string())));
                                    return None;
                                },
                            }
                        },
                        None => {
                            Log::push_stop(log, 13, None);
                            return None;
                        },
                    };
                },
                "start" => mode = Mode::Start,
                "stop" => mode = Mode::Stop,
                "go" => mode = Mode::Go,
                _ => {},
            },
        };
        if !conf_found {
            conf = match args.next() {
                Some(c) => if c.as_str() == "-r" {
                    match args.next() {
                        Some(p) => {
                            let file = format!("{}/tiny.conf", p);
                            match read_to_string(&file) {
                                Ok(s) => {
                                    conf_file = file;
                                    Some(s)
                                },
                                Err(e) => {
                                    Log::push_stop(log, 14, Some(format!("{}. Error: {}", &p, e.to_string())));
                                    return None;
                                },
                            }
                        },
                        None => {
                            Log::push_stop(log, 13, None);
                            return None;
                        },
                    }
                } else {
                    None
                },
                None => None,
            }
        };
        let exe_path = match exe_file.rfind('/') {
            Some(i) => exe_file[..i].to_owned(),
            None => {
                Log::push_stop(log, 16, Some(exe_file));
                return None;
            },
        };
        if let None = conf {
            let file = format!("{}/tiny.conf", exe_path);
            conf = match read_to_string(&file) {
                Ok(s) => {
                    conf_file= file;
                    Some(s)
                },
                Err(e) => match e.kind() {
                    ErrorKind::NotFound => None,
                    _ => {
                        Log::push_stop(log, 14, Some(format!("{}. Error: {}", &file, e.to_string())));
                        return None;
                    },
                },
            };
        };
        let conf = match conf {
            Some(c) => c,
            None => {
                let file = match env::current_dir() {
                    Ok(f) => match f.to_str() {
                        Some(s) => format!("{}/tiny.conf", s.replace("\\", "/")),
                        None => {
                            Log::push_stop(log, 15, None);
                            return None;
                        },
                    },
                    Err(_) => {
                        Log::push_stop(log, 15, None);
                        return None;
                    },
                };
                match read_to_string(&file) {
                    Ok(s) => {
                        conf_file = file;
                        s
                    },
                    Err(_) => {
                        Log::push_stop(log, 15, None);
                        return None;
                    },
                }
            },
        };
        let root_path = match conf_file.rfind('/') {
            Some(i) => conf_file[..i].to_owned(),
            None => {
                Log::push_stop(log, 16, Some(conf_file));
                return None;
            },
        };

        let conf = match Init::load_conf(log, conf) {
            Some(c) => c,
            None => return None,
        };

        Some(Init {
            mode,
            conf,
            exe_file,
            exe_path,
            conf_file,
            root_path,
        })
    }

    fn load_conf(log: Arc<Mutex<Log>>, text: String) -> Option<Config>{
        let json: Result<Value, serde_json::Error> = serde_json::from_str(&text);
        match json {
            Ok(json) => {
                match json.get("log") {
                    Some(v) => match v.as_str() {
                        Some(s) => match OpenOptions::new().create(true).write(true).append(true).open(s) {
                            Ok(_) => {
                                Log::set_path(Arc::clone(&log), s.to_owned());
                            },
                            Err(e) => {
                                Log::push_stop(log, 53, Some(format!("Can't create file {}. Error: {}", s, e.to_string())));
                                return None;
                            },
                        },
                        None => {
                            Log::push_stop(log, 52, None);
                            return None;
                        },
                    },
                    None => {
                        Log::push_stop(log, 51, None);
                        return None;
                    },
                };
                let version = match json.get("version") {
                    Some(v) => match v.as_str() {
                        Some(s) => s.to_owned(),
                        None => {
                            Log::push_stop(log, 55, None);
                            return None;
                        },
                    },
                    None => {
                        Log::push_stop(log, 54, None);
                        return None;
                    },
                };
                let max = match json.get("max") {
                    Some(v) => match v.as_i64() {
                        Some(s) => match u8::try_from(s) {
                            Ok(m) => m,
                            Err(e) => {
                                Log::push_stop(log, 58, Some(e.to_string()));
                                return None;
                            },
                        },
                        None => {
                            Log::push_stop(log, 57, None);
                            return None;
                        },
                    },
                    None => {
                        Log::push_stop(log, 56, None);
                        return None;
                    },
                };
                let bind_port = match json.get("port") {
                    Some(v) => match v.as_i64() {
                        Some(s) => match u16::try_from(s) {
                            Ok(m) => m,
                            Err(e) => {
                                Log::push_stop(log, 61, Some(e.to_string()));
                                return None;
                            },
                        },
                        None => {
                            Log::push_stop(log, 60, None);
                            return None;
                        },
                    },
                    None => {
                        Log::push_stop(log, 59, None);
                        return None;
                    },
                };
                let bind_ip = match json.get("ip") {
                    Some(v) => match v.as_str() {
                        Some(s) => match IpAddr::from_str(s) {
                            Ok(i) => i,
                            Err(e) => {
                                Log::push_stop(log, 64, Some(format!("ip in file={}. Error: {}", s, e.to_string())));
                                return None;
                            },
                        },
                        None => {
                            Log::push_stop(log, 63, None);
                            return None;
                        },
                    },
                    None => {
                        Log::push_stop(log, 62, None);
                        return None;
                    },
                };
                let bind_accept = match json.get("accept") {
                    Some(v) => match v.as_str() {
                        Some(s) => match IpAddr::from_str(s) {
                            Ok(i) => i,
                            Err(e) => {
                                Log::push_stop(log, 93, Some(format!("ip in file={}. Error: {}", s, e.to_string())));
                                return None;
                            },
                        },
                        None => {
                            Log::push_stop(log, 92, None);
                            return None;
                        },
                    },
                    None => {
                        Log::push_stop(log, 91, None);
                        return None;
                    },
                };
                let rpc_port = match json.get("rpc_port") {
                    Some(v) => match v.as_i64() {
                        Some(s) => match u16::try_from(s) {
                            Ok(m) => m,
                            Err(e) => {
                                Log::push_stop(log, 67, Some(e.to_string()));
                                return None;
                            },
                        },
                        None => {
                            Log::push_stop(log, 66, None);
                            return None;
                        },
                    },
                    None => {
                        Log::push_stop(log, 65, None);
                        return None;
                    },
                };
                let rpc_accept = match json.get("rpc_accept") {
                    Some(v) => match v.as_str() {
                        Some(s) => match IpAddr::from_str(s) {
                            Ok(i) => i,
                            Err(e) => {
                                Log::push_stop(log, 90, Some(format!("ip in file={}. Error: {}", s, e.to_string())));
                                return None;
                            },
                        },
                        None => {
                            Log::push_stop(log, 89, None);
                            return None;
                        },
                    },
                    None => {
                        Log::push_stop(log, 88, None);
                        return None;
                    },
                };
                let rpc_ip = match json.get("rpc_ip") {
                    Some(v) => match v.as_str() {
                        Some(s) => match IpAddr::from_str(s) {
                            Ok(i) => i,
                            Err(e) => {
                                Log::push_stop(log, 70, Some(format!("ip in file={}. Error: {}", s, e.to_string())));
                                return None;
                            },
                        },
                        None => {
                            Log::push_stop(log, 69, None);
                            return None;
                        },
                    },
                    None => {
                        Log::push_stop(log, 68, None);
                        return None;
                    },
                };
                let zone = match json.get("zone") {
                    Some(v) => match v.as_str() {
                        Some(s) => s.to_owned(),
                        None => {
                            Log::push_stop(log, 72, None);
                            return None;
                        },
                    },
                    None => {
                        Log::push_stop(log, 71, None);
                        return None;
                    },
                };
                let salt = match json.get("salt") {
                    Some(v) => match v.as_str() {
                        Some(s) => s.to_owned(),
                        None => {
                            Log::push_stop(log, 74, None);
                            return None;
                        },
                    },
                    None => {
                        Log::push_stop(log, 73, None);
                        return None;
                    },
                };
                let lang_id = match json.get("lang") {
                    Some(v) => match v.as_i64() {
                        Some(s) => match u8::try_from(s) {
                            Ok(m) => m,
                            Err(e) => {
                                Log::push_stop(log, 96, Some(e.to_string()));
                                return None;
                            },
                        },
                        None => {
                            Log::push_stop(log, 95, None);
                            return None;
                        },
                    },
                    None => {
                        Log::push_stop(log, 94, None);
                        return None;
                    },
                };
                let db = match json.get("db") {
                    Some(v) => match v.as_object() {
                        Some(db) => {
                            let host = match db.get("host") {
                                Some(v) => match v.as_str() {
                                    Some(s) => s.to_owned(),
                                    None => {
                                        Log::push_stop(log, 78, None);
                                        return None;
                                    },
                                },
                                None => {
                                    Log::push_stop(log, 77, None);
                                    return None;
                                },
                            };
                            let port = match db.get("port") {
                                Some(v) => match v.as_i64() {
                                    Some(s) => match u16::try_from(s) {
                                        Ok(m) => m,
                                        Err(e) => {
                                            Log::push_stop(log, 81, Some(e.to_string()));
                                            return None;
                                        },
                                    },
                                    None => {
                                        Log::push_stop(log, 80, None);
                                        return None;
                                    },
                                },
                                None => {
                                    Log::push_stop(log, 79, None);
                                    return None;
                                },
                            };
                            let name = match db.get("name") {
                                Some(v) => match v.as_str() {
                                    Some(s) => s.to_owned(),
                                    None => {
                                        Log::push_stop(log, 83, None);
                                        return None;
                                    },
                                },
                                None => {
                                    Log::push_stop(log, 82, None);
                                    return None;
                                },
                            };
                            let user = match db.get("user") {
                                Some(v) => match v.as_str() {
                                    Some(s) => s.to_owned(),
                                    None => {
                                        Log::push_stop(log, 85, None);
                                        return None;
                                    },
                                },
                                None => {
                                    Log::push_stop(log, 84, None);
                                    return None;
                                },
                            };
                            let pwd = match db.get("pwd") {
                                Some(v) => match v.as_str() {
                                    Some(s) => s.to_owned(),
                                    None => {
                                        Log::push_stop(log, 87, None);
                                        return None;
                                    },
                                },
                                None => {
                                    Log::push_stop(log, 86, None);
                                    return None;
                                },
                            };
                            DBConfig { host, port, name, user, pwd }
                        },
                        None => {
                            Log::push_stop(log, 76, None);
                            return None;
                        },
                    },
                    None => {
                        Log::push_stop(log, 75, None);
                        return None;
                    },
                };
                Some(Config {
                    version,
                    max,
                    bind_accept,
                    bind_port,
                    bind_ip,
                    rpc_port,
                    rpc_ip,
                    rpc_accept,
                    zone,
                    salt,
                    lang_id,
                    db,
                })
            },
            Err(e) => {
                Log::push_stop(log, 50, Some(e.to_string()));
                return None;
            },
        }
    }

}