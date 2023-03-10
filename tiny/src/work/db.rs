use std::{sync::{Arc, Mutex}, fmt};

use native_tls::Protocol;
use postgres::{Client, Row, types::{ToSql, Type}, Statement, ToStatement};
use postgres_native_tls::MakeTlsConnector;

use crate::sys::log::Log;

use super::{cache::Cache, action::Data};

#[derive(Debug, Clone)]
pub struct DBConfig {
    pub host: String,
    pub port: u16,
    pub name: String,
    pub user: String,
    pub pwd: String
}

enum DBResult {
    Ok(Vec<Row>),
    ErrQuery(String),
    ErrConnect(String),
}

pub struct DB {
    sql: Option<Client>,
    config: DBConfig,
    pub error: Option<String>,
    log: Arc<Mutex<Log>>,
    timezone: String,
    pub prepare: Vec<(Statement, &'static str)>,
    cache: Arc<Mutex<Cache>>,
}

impl fmt::Debug for DB {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let sql = match &self.sql {
            Some(c) => if c.is_closed() {
                "connection is closed"
            } else {
                "connection is ok"
            },
            None => "connection is empty",
        };

        f.debug_struct("DB")
         .field("sql", &sql)
         .field("config", &self.config)
         .field("error", &self.error)
         .field("timezone", &self.timezone)
         .finish()
    }
}

impl DB {
    pub fn one_time_query(config: &DBConfig, log: Arc<Mutex<Log>>, query: &str) -> Option<Vec<Row>> {
        let connector = match native_tls::TlsConnector::builder().danger_accept_invalid_certs(true).min_protocol_version(Some(Protocol::Tlsv12)).build() {
            Ok(c) => c,
            Err(e) => {
                Log::push_warning(log, 600, Some(e.to_string()));
                return None;
            },
        };
        let builder = MakeTlsConnector::new(connector);
        let conn_str = format!("host='{}' port='{}' dbname='{}' user='{}' password='{}' sslmode=require connect_timeout=2 application_name='{} {}' options='--client_encoding=UTF8'", config.host, config.port, config.name, config.user, config.pwd, &env!("CARGO_PKG_NAME"), &env!("CARGO_PKG_VERSION"));

        let mut sql = match Client::connect(&conn_str, builder) {
            Ok(sql) => sql,
            Err(e) => {
                Log::push_warning(log, 601, Some(e.to_string()));
                return None;
            },
        };
        match DB::exec(&mut sql, query, &[]) {
            DBResult::Ok(r) => Some(r),
            _ => None,
        }
    }

    pub fn new(config: DBConfig, log: Arc<Mutex<Log>>, timezone: String, cache: Arc<Mutex<Cache>>) -> DB {
        match DB::connect(&config, Arc::clone(&log), &timezone, Arc::clone(&cache)) {
            Ok((db, prepare)) => {
                DB {
                    sql: Some(db),
                    config,
                    error: None,
                    log,
                    timezone,
                    prepare,
                    cache,
                }
            },
            Err(e) => {
                Log::push_warning(Arc::clone(&log), 603, Some(e.clone()));
                DB {
                    sql: None,
                    config,
                    error: Some(e),
                    log,
                    timezone,
                    prepare: Vec::new(),
                    cache,
                }
            },
        }
    }

    fn connect(config: &DBConfig, log: Arc<Mutex<Log>>, timezone: &str, cache: Arc<Mutex<Cache>>) -> Result<(Client, Vec<(Statement, &'static str)>), String> {
        let connector = match native_tls::TlsConnector::builder().danger_accept_invalid_certs(true).min_protocol_version(Some(Protocol::Tlsv12)).build() {
            Ok(c) => c,
            Err(e) => {
                Log::push_warning(log, 600, Some(e.to_string()));
                return Err(e.to_string());
            },
        };
        let builder = MakeTlsConnector::new(connector);
        let conn_str = format!("host='{}' port='{}' dbname='{}' user='{}' password='{}' sslmode=require connect_timeout=2 application_name='{} {}' options='--client_encoding=UTF8'", config.host, config.port, config.name, config.user, config.pwd, &env!("CARGO_PKG_NAME"), &env!("CARGO_PKG_VERSION"));

        let mut sql = match Client::connect(&conn_str, builder) {
            Ok(sql) => sql,
            Err(e) => {
                Log::push_warning(log, 601, Some(e.to_string()));
                return Err(e.to_string());
            },
        };

        let query = format!("SET timezone TO '{}';", timezone);
        if let Err(e) = sql.query(&query, &[]) {
            Log::push_warning(log, 602, Some(format!("{} error={} {}", query, e.to_string(), timezone)));
            return Err(e.to_string());
        };
        let prepare = DB::prepare(&mut sql, Arc::clone(&log), Arc::clone(&cache));
        Ok((sql, prepare))
    }

    pub fn is_not_empty(&self) -> bool {
        match self.sql {
            Some(_) => true,
            None => false,
        }
    }

    pub fn check(&mut self) {
        let close = match &self.sql {
            Some(c) => c.is_closed(),
            None => true,
        };
        if close {
            match DB::connect(&self.config, Arc::clone(&self.log), &self.timezone, Arc::clone(&self.cache)) {
                Ok((db, prepare)) => {
                    self.sql = Some(db);
                    self.prepare = prepare;
                    self.error = None;
                },
                Err(e) => {
                    self.sql = None;
                    self.error = Some(e);
                    self.prepare = Vec::new();
                },
            };
        }
    }

    pub fn query_fast(&mut self, index: usize, params: &[&(dyn ToSql + Sync)]) -> Option<Vec<Row>> {
        match &mut self.sql {
            Some(c) => {
                let (statement, source)= match self.prepare.get(index) {
                    Some(s) => s,
                    None => return None,
                };
                match DB::exec(c, statement, params) {
                    DBResult::Ok(r) => {
                        self.error = None;
                        Some(r)
                    },
                    DBResult::ErrQuery(e) => {
                        Log::push_warning(Arc::clone(&self.log), 602, Some(format!("{} error={}", source, e.clone())));
                        self.error = Some(e);
                        None
                    },
                    DBResult::ErrConnect(e) => {
                        self.sql = None;
                        self.prepare = Vec::new();
                        self.error = Some(e);
                        None
                    },
                }
            },
            None => None,
        }
    }

    pub fn query_params(&mut self, query: &str, params: &[&(dyn ToSql + Sync)]) -> Option<Vec<Row>> {
        match &mut self.sql {
            Some(c) => match DB::exec(c, query, params) {
                DBResult::Ok(r) => {
                    self.error = None;
                    Some(r)
                },
                DBResult::ErrQuery(e) => {
                    Log::push_warning(Arc::clone(&self.log), 602, Some(format!("{} error={}", query, e.clone())));
                    self.error = Some(e);
                    None
                },
                DBResult::ErrConnect(e) => {
                    self.sql = None;
                    self.prepare = Vec::new();
                    self.error = Some(e);
                    None
                },
            },
            None => None,
        }
    }
    
    pub fn query(&mut self, query: &str) -> Option<Vec<Row>> {
        match &mut self.sql {
            Some(c) => match DB::exec(c, query, &[]) {
                DBResult::Ok(r) => {
                    self.error = None;
                    Some(r)
                },
                DBResult::ErrQuery(e) => {
                    Log::push_warning(Arc::clone(&self.log), 602, Some(format!("{} error={}", query, e.clone())));
                    self.error = Some(e);
                    None
                },
                DBResult::ErrConnect(e) => {
                    self.sql = None;
                    self.prepare = Vec::new();
                    self.error = Some(e);
                    None
                },
            },
            None => None,
        }
    }

    fn exec<T>(sql: &mut Client, query: &T, params: &[&(dyn ToSql + Sync)]) -> DBResult 
    where
        T: ?Sized + ToStatement,
    {
        match sql.query(query, params) {
            Ok(res) => DBResult::Ok(res),
            Err(e) => if e.is_closed() {
                DBResult::ErrConnect(e.to_string())
            } else {
                DBResult::ErrQuery(e.to_string())
            },
        }
    }
    
    fn prepare(db: &mut Client, log: Arc<Mutex<Log>>, cache: Arc<Mutex<Cache>>) -> Vec<(Statement, &'static str)> {
        let mut vec = Vec::with_capacity(64);
        // 0 Get / Insert session
        let sql = "
            WITH 
            new_q AS (
                SELECT 0::int8 user_id, $1::text session, '\\x'::bytea data, now() created, now() last, $2 ip, $3 user_agent, $4 lang_id
            ),
            ins_q AS (
                INSERT INTO session (user_id, session, data, created, last, ip, user_agent, lang_id) 
                SELECT n.user_id, n.session, n.data, n.created, n.last, n.ip, n.user_agent, n.lang_id
                FROM 
                new_q n
                LEFT JOIN session s ON s.session=n.session
                WHERE s.session_id IS NULL
                RETURNING session_id, data, user_id, lang_id
            ),
            res AS (
                SELECT session_id, data, user_id, lang_id FROM ins_q
                UNION 
                SELECT session_id, data, user_id, lang_id FROM session WHERE session=$5
            )
            SELECT r.session_id, r.user_id, u.role_id, r.data, r.lang_id FROM res r INNER JOIN \"user\" u ON u.user_id=r.user_id
        ";
        match db.prepare_typed(sql, &[Type::TEXT, Type::TEXT, Type::TEXT, Type::INT8, Type::TEXT]) {
            Ok(s) => {
                vec.push((s, sql));
            },
            Err(e) => Log::push_error(log, 604, Some(e.to_string())),
        };
        
        // 1 Update session
        let sql = "
            UPDATE session
            SET 
                user_id=$1,
                data=$2,
                last=now(),
                ip=$3,
                user_agent=$4,
                lang_id=$5
            WHERE
                session_id=$6
        ";
        match db.prepare_typed(sql, &[Type::INT8, Type::BYTEA, Type::TEXT, Type::TEXT, Type::INT8, Type::INT8]) {
            Ok(s) => {
                vec.push((s, sql));
            },
            Err(e) => Log::push_error(log, 604, Some(e.to_string())),
        };

        // 2 Update session
        let sql = "
            UPDATE session 
            SET 
                last = now()
            WHERE
                session_id=$1
        ";
        match db.prepare_typed(sql, &[Type::INT8]) {
            Ok(s) => {
                vec.push((s, sql));
            },
            Err(e) => Log::push_error(log, 604, Some(e.to_string())),
        };

        // 3 Get auth permissions
        let sql = "
            SELECT COALESCE(MAX(a.access::int), 0)::bool AS access
            FROM 
                access a
                INNER JOIN \"user\" u ON u.role_id=a.role_id
                INNER JOIN controller c ON a.controller_id=c.controller_id
            WHERE 
                a.access AND u.user_id=$1 AND (
                    (c.module='' AND c.class='' AND c.action='')
                    OR (c.module=$2 AND c.class='' AND c.action='')
                    OR (c.module=$3 AND c.class=$5 AND c.action='')
                    OR (c.module=$4 AND c.class=$6 AND c.action=$7)
                )
        ";
        match db.prepare_typed(sql, &[Type::INT8, Type::TEXT, Type::TEXT, Type::TEXT, Type::TEXT, Type::TEXT, Type::TEXT]) {
            Ok(s) => {
                vec.push((s, sql));
            },
            Err(e) => Log::push_error(log, 604, Some(e.to_string())),
        };

        // 4 Get url by route map
        let sql = "
            SELECT r.url
            FROM 
                route r
                INNER JOIN controller c ON c.controller_id=r.controller_id
            WHERE c.module=$1 AND c.class=$2 AND c.action=$3 AND COALESCE(r.params, '')=$4 AND COALESCE(r.lang_id, -1)=$5
        ";
        match db.prepare_typed(sql, &[Type::TEXT, Type::TEXT, Type::TEXT, Type::TEXT, Type::INT8]) {
            Ok(s) => {
                vec.push((s, sql));
            },
            Err(e) => Log::push_error(log, 604, Some(e.to_string())),
        };

        DB::load_db_cache(db, log, cache);
        vec
    }

    fn load_db_cache(db: &mut Client, log: Arc<Mutex<Log>>, cache: Arc<Mutex<Cache>>) {
        let sql = "
            SELECT url, redirect, permanently FROM redirect
        ";
        Cache::del(Arc::clone(&cache), "redirect", Arc::clone(&log));
        if let DBResult::Ok(res) = DB::exec(db, sql, &[]) {
            let mut url: String;
            let mut key: String;
            let mut redirect: String;
            let mut permanently: bool;
            let mut value: String;
            for row in res {
                url = row.get(0);
                key = format!("redirect:{}", &url);
                redirect = row.get(1);
                permanently = row.get(2);
                value = if permanently {
                    format!("1{}", &redirect)
                } else {
                    format!("0{}", &redirect)
                };
                Cache::set(Arc::clone(&cache), key, Data::String(value), Arc::clone(&log));
            }
        };

        let sql = "
            SELECT r.url, c.module, c.class, c.action, r.params, r.lang_id
            FROM route r INNER JOIN controller c ON r.controller_id=c.controller_id
            WHERE LENGTH(c.module)>0 AND LENGTH(c.class)>0 AND LENGTH(c.action)>0
        ";
        Cache::del(Arc::clone(&cache), "route", Arc::clone(&log));
        Cache::del(Arc::clone(&cache), "404", Arc::clone(&log));
        if let DBResult::Ok(res) = DB::exec(db, sql, &[]) {
            let mut url: String;
            let mut key: String;
            let mut module: String;
            let mut class: String;
            let mut action: String;
            let mut param: Option<String>;
            let mut lang_id: Option<i64>;
            let mut data = Vec::with_capacity(5);

            for row in res {
                url = row.get(0);
                key = format!("route:{}", &url);
                module = row.get(1);
                class = row.get(2);
                action = row.get(3);
                param = row.get(4);
                lang_id = row.get(5);
                data.clear();
                if &module == "index" && &class == "index" && action == "not_found" {
                    match &lang_id {
                        Some(i) => Cache::set(Arc::clone(&cache), format!("404:{}", *i), Data::String(url), Arc::clone(&log)),
                        None => Cache::set(Arc::clone(&cache), "404".to_owned(), Data::String(url), Arc::clone(&log)),
                    };
                }
                data.push(Data::String(module));
                data.push(Data::String(class));
                data.push(Data::String(action));
                match &param {
                    Some(s) => data.push(Data::String(s.clone())),
                    None => data.push(Data::None),
                };
                match &lang_id {
                    Some(i) => data.push(Data::U64(*i as u64)),
                    None => data.push(Data::None),
                };

                Cache::set(Arc::clone(&cache), key, Data::Vec(data.clone()), Arc::clone(&log));
            }
        }
    }

}
