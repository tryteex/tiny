use std::{sync::{Arc, Mutex, mpsc::{self, Sender, Receiver}, atomic::{AtomicBool, Ordering}, RwLock}, net::{TcpListener, SocketAddr, TcpStream, IpAddr, Ipv4Addr, Shutdown}, time::Duration, io::{Read, Write}, thread::{self, JoinHandle}, process, collections::HashMap};

use crate::{work::{worker::{Worker, MessageWork}, cache::Cache, db::DBConfig, action::{ActMap, Act}, html::Html, lang::Lang}};

use super::{log::Log, app::App, init::Mode};

const ANY_IP: IpAddr = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0));

pub struct Go {}

impl Go {
    
    fn get_engine() -> ActMap{
        tinymod::addfn!();
    }

    pub fn run(app: App) {
        let log = Arc::clone(&app.log);
        let html = match Html::new(&app.init.root_path, Arc::clone(&log)) {
            Some(h) => Arc::new(RwLock::new(h)),
            None => return,
        };
        let lang = Arc::new(RwLock::new(Lang::new(&app.init.root_path, &app.init.conf.db, Arc::clone(&log))));
        let bind_socket = SocketAddr::new(app.init.conf.bind_ip, app.init.conf.bind_port);
        let bind = match TcpListener::bind(&bind_socket) {
            Ok(i) => i,
            Err(e) => {
                Log::push_stop(log, 500, Some(e.to_string()));
                return;
            },
        };
        let bind_accept = if app.init.conf.bind_accept != ANY_IP {
            Some(app.init.conf.rpc_accept)
        } else {
            None
        };
        let irc = match TcpListener::bind(SocketAddr::new(app.init.conf.rpc_ip, app.init.conf.rpc_port)) {
            Ok(i) => i,
            Err(e) => {
                Log::push_stop(log, 202, Some(e.to_string()));
                return;
            },
        };
        
        let stop = Arc::new(AtomicBool::new(false));

        let cache = Cache::new();
        let (sender, receiver) = mpsc::channel();
        let engine = Go::get_engine();
        let (workers, senders_work, receiver_ready) = Go::start_worker(app.init.conf.max, cache, html, lang, Arc::clone(&log), app.init.conf.salt.clone(), app.init.root_path.clone(), app.init.conf.db.clone(), app.init.conf.zone.clone(), app.init.conf.lang_id, engine);
        let run = Go::run_worker(receiver_ready, Arc::clone(&senders_work), Arc::clone(&stop), Arc::clone(&log), receiver);

        let main = Go::wait_incoming(&app.init.conf.salt, bind, bind_accept, Arc::clone(&app.log), sender);

        Go::listen_rpc(irc, app, workers, senders_work, run, stop, log, main, &bind_socket);

    }

    fn start_worker<'a>(
        max: u8, 
        cache: Arc<Mutex<Cache>>, 
        html: Arc<RwLock<Html>>, 
        lang: Arc<RwLock<Lang>>, 
        log: Arc<Mutex<Log>>, 
        salt: String, 
        path: String, 
        db: DBConfig, 
        timezone: String, 
        lang_id: u8, 
        engine: ActMap
    ) -> (Vec<Worker>, Arc<Mutex<Vec<Sender<MessageWork>>>>, Receiver<u8>) {
        let mut workers = Vec::with_capacity(max as usize);
        let mut senders_work = Vec::with_capacity(max as usize);
        let (sender_ready, receiver_ready) = mpsc::channel();
        let sender_ready = Arc::new(Mutex::new(sender_ready));
        for i in 0..max {
            let (sender_work, receiver_work) = mpsc::channel();
            workers.push(Worker::new(i, receiver_work, Arc::clone(&sender_ready), Arc::clone(&cache), Arc::clone(&html), Arc::clone(&lang), Arc::clone(&log), salt.clone(), path.clone(), db.clone(), timezone.clone(), lang_id, engine.clone()));
            senders_work.push(sender_work);
        }
        (workers, Arc::new(Mutex::new(senders_work)), receiver_ready)
    }

    fn run_worker(receivers_ready: Receiver<u8>, senders_work: Arc<Mutex<Vec<Sender<MessageWork>>>>, stop: Arc<AtomicBool>, log: Arc<Mutex<Log>>, receiver: Receiver<MessageWork>) -> JoinHandle<()> {
        thread::spawn(move || {
            loop {
                match receiver.recv() {
                    Ok(m) => match m {
                        MessageWork::Terminate => break,
                        MessageWork::Job(tcp) => {
                            match receivers_ready.recv() {
                                Ok(ind) => {
                                    if stop.load(Ordering::Acquire) {
                                        continue;
                                    }
                                    match Mutex::lock(&senders_work) {
                                        Ok(senders) => {
                                            let sender = match senders.get(ind as usize) {
                                                Some(s) => s,
                                                None => Log::push_error(log, 704, None),
                                            };
                                            if let Err(e) = sender.send(MessageWork::Job(tcp)) {
                                                Log::push_error(log, 705, Some(e.to_string()));
                                            }
                                        },
                                        Err(e) => Log::error(log, e.to_string()),
                                    };
                                },
                                Err(e) => Log::push_error(log, 706, Some(e.to_string())),
                            }
                        },
                    },
                    Err(e) => Log::push_error(log, 511, Some(e.to_string())),
                };
            }
        })
    }

    fn wait_incoming(salt: &str, bind: TcpListener, accept: Option<IpAddr>, log: Arc<Mutex<Log>>, sender: Sender<MessageWork>) -> JoinHandle<()> {
        let uuid_stop = format!("stop {}", salt).as_bytes().to_vec();
        thread::spawn(move || {

            let mut uuid_len;
            let uuid_stop_len = uuid_stop.len();
            let mut uuid_buf = vec![0; uuid_stop_len];

            for stream in bind.incoming() {
                let tcp = match stream {
                    Ok(s) => s,
                    Err(e) => {
                        Log::push_warning(Arc::clone(&log), 502, Some(e.to_string()));
                        continue;
                    },
                };
                if let Some(a) = accept {
                    let addr = match tcp.peer_addr() {
                        Ok(a) => a,
                        Err(e) => {
                            Log::push_warning(Arc::clone(&log), 503, Some(e.to_string()));
                            continue;
                        },
                    };

                    if addr.ip() != a {
                        Log::push_warning(Arc::clone(&log), 504, Some(format!("{}", addr)));
                        continue;
                    }
                }

                uuid_len = match tcp.peek(&mut uuid_buf) {
                    Ok(l) => l,
                    Err(e) => {
                        Log::push_warning(Arc::clone(&log), 505, Some(e.to_string()));
                        continue;
                    },
                };
                if uuid_len == uuid_stop_len && &uuid_buf[..uuid_len] == &uuid_stop {
                    if let Err(e) = sender.send(MessageWork::Terminate) {
                        Log::push_error(log, 512, Some(e.to_string()));
                    };
                    break;
                }
                if let Err(e) = sender.send(MessageWork::Job(tcp)) {
                    Log::push_error(log, 510, Some(e.to_string()));
                };
            }
        })
    }

    fn listen_rpc(irc: TcpListener, app: App, workers: Vec<Worker>, sends: Arc<Mutex<Vec<Sender<MessageWork>>>>, run: JoinHandle<()>, stop: Arc<AtomicBool>, log: Arc<Mutex<Log>>, main: JoinHandle<()>, stop_socket: &SocketAddr) {
        let rpc_accept = if app.init.conf.rpc_accept != ANY_IP {
            Some(app.init.conf.rpc_accept)
        } else {
            None
        };
        let stop_data = format!("stop {}", app.init.conf.salt);
        for stream in irc.incoming() {
            match stream {
                Ok(mut s) => if let Some(m) = Go::get_rpc_connect(&mut s, Arc::clone(&log), &rpc_accept, &stop_data) {
                    match m {
                        Mode::Stop => {
                            Go::stop(workers, sends, Arc::clone(&log), run, stop, main, stop_socket, &stop_data);
                            if let Err(e) = s.write_all(format!("stop {}", process::id()).as_bytes()) {
                                Log::push_warning(log, 216, Some(e.to_string()));
                            };
                            break;
                        },
                        _ => {},
                    };
                },
                Err(e) => {
                    Log::push_warning(Arc::clone(&log), 203, Some(e.to_string()));
                },
            }
        }
    }

    fn get_rpc_connect(tcp: &mut TcpStream, log: Arc<Mutex<Log>>, rpc_accept: &Option<IpAddr>, stop: &str) -> Option<Mode> {
        if let Some(a) = rpc_accept {
            let addr = match tcp.peer_addr() {
                Ok(a) => a,
                Err(e) => {
                    Log::push_warning(log, 208, Some(e.to_string()));
                    return None;
                },
            };

            if &addr.ip() == a {
                Log::push_info(Arc::clone(&log), 209, Some(format!("{}", addr)));
            } else {
                Log::push_warning(log, 210, Some(format!("{}", addr)));
                return None;
            }
        }

        if let Err(e) = tcp.set_read_timeout(Some(Duration::new(3, 0))) {
            Log::push_warning(log, 204, Some(e.to_string()));
            return None;
        };
        let mut buf = [0; 1024];
        let len = match tcp.read(&mut buf) {
            Ok(l) => l,
            Err(e) => {
                Log::push_warning(log, 205, Some(e.to_string()));
                return None;
            },
        };
        if &buf[..len] == stop.as_bytes() {
            Log::push_info(log, 207, None);
            return Some(Mode::Stop);
        } 
        Log::push_warning(log, 206, Some(format!("{:x?}", &buf[..len])));
        return None;
    }

    fn stop(workers: Vec<Worker>, senders_work: Arc<Mutex<Vec<Sender<MessageWork>>>>, log: Arc<Mutex<Log>>, run: JoinHandle<()>, stop: Arc<AtomicBool>, main: JoinHandle<()>, stop_socket: &SocketAddr, stop_data: &str) {
        match TcpStream::connect_timeout(stop_socket, Duration::from_secs(1)) {
            Ok(mut tcp) => {
                if let Err(e) = tcp.write_all(stop_data.as_bytes()) {
                    Log::push_error(log, 507, Some(e.to_string()));
                };
                if let Err(e) = tcp.shutdown(Shutdown::Both) {
                    Log::push_warning(Arc::clone(&log), 508, Some(e.to_string()));
                };
            },
            Err(e) => {
                Log::push_error(log, 506, Some(e.to_string()));
            },
        };

        if let Err(e) = main.join() {
            match (e.downcast_ref::<&str>(), e.downcast_ref::<String>()) {
                (Some(&e), _) => Log::push_error(log, 501, Some(e.to_owned())),
                (_, Some(e)) => Log::push_error(log, 501, Some(e.to_owned())),
                (None, None) =>  Log::push_error(log, 501, None),
            };
        };

        stop.store(true, Ordering::Release);

        if let Err(e) = run.join() {
            match (e.downcast_ref::<&str>(), e.downcast_ref::<String>()) {
                (Some(&e), _) => Log::push_error(log, 509, Some(e.to_owned())),
                (_, Some(e)) => Log::push_error(log, 509, Some(e.to_owned())),
                (None, None) =>  Log::push_error(log, 509, None),
            };
        };

        match Mutex::lock(&senders_work) {
            Ok(sender_work) => {
                for s in sender_work.iter() {
                    if let Err(e) = s.send(MessageWork::Terminate) {
                        Log::push_error(log, 701, Some(e.to_string()));
                    };
                }
            },
            Err(e) => Log::error(log, e.to_string()),
        };

        for w in workers {
            if let Err(e) = w.join() {
                match (e.downcast_ref::<&str>(), e.downcast_ref::<String>()) {
                    (Some(&e), _) => Log::push_error(log, 702, Some(e.to_owned())),
                    (_, Some(e)) => Log::push_error(log, 702, Some(e.to_owned())),
                    (None, None) =>  Log::push_error(log, 702, None),
                };
            };
        }
    }

}
