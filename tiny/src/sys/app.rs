use std::io::{Write, Read};
use std::net::{TcpStream, SocketAddr};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::help::Help;

use super::{log::{Log}, init::{Init, Mode}, go::Go};

#[derive(Debug)]
pub struct App {
    pub log: Arc<Mutex<Log>>,
    pub init: Init,
}

impl App {
    pub fn new(log: Arc<Mutex<Log>>) -> Option<App> {
        let init = match Init::new(Arc::clone(&log)) {
            Some(i) => i,
            None => return None,
        };
        Some(App {
            log,
            init,
        })
    }

    pub fn get_mode(&self) -> Mode{
        self.init.mode
    }

    pub fn run(app: App) {
        match app.init.mode {
            Mode::Start => App::start(app),
            Mode::Stop => App::stop(app),
            Mode::Help => Help::show(),
            Mode::Go => Go::run(app),
        };
    }

    fn stop(app: App) {
        let mut tcp = match TcpStream::connect_timeout(&SocketAddr::new(app.init.conf.rpc_ip, app.init.conf.rpc_port), Duration::from_secs(1)) {
            Ok(t) => t,
            Err(e) => {
                Log::push_stop(app.log, 213, Some(e.to_string()));
                return;
            },
        };
        if let Err(e) = tcp.write_all(format!("stop {}", app.init.conf.salt).as_bytes()) {
            Log::push_stop(app.log, 214, Some(e.to_string()));
            return;
        };
        if let Err(e) = tcp.set_read_timeout(Some(Duration::from_secs(30))) {
            Log::push_stop(app.log, 217, Some(e.to_string()));
            return;
        };

        let mut buf: [u8; 1024] = [0; 1024];
        let s = match tcp.read(&mut buf) {
            Ok(s) => s,
            Err(e) => {
                Log::push_stop(app.log, 218, Some(e.to_string()));
                return;
            },
        };
        if s < 6 {
            Log::push_stop(app.log, 219, Some(format!("{:?}", &buf[..s])));
            return;
        }
        let pid = match String::from_utf8((&buf[5..s]).to_vec()) {
            Ok(i) => i,
            Err(e) => {
                Log::push_stop(app.log, 220, Some(format!("{:?} {}", &buf[5..s], e.to_string())));
                return;
            },
        };
        Log::push_info(app.log, 215, Some(format!("Answer PID={}", pid)));
    }

    #[cfg(target_family="windows")]
    fn start(app: App) {
        let path = App::to_win_path(&app.init.exe_path);
        let exe = App::to_win_path(&app.init.exe_file);
        const DETACHED_PROCESS: u32 = 0x00000008;
        const CREATE_NEW_PROCESS_GROUP: u32 = 0x00000200;
        const CREATE_NO_WINDOW: u32 = 0x08000000;

        let args = ["go", "-r", &app.init.root_path];
        use std::os::windows::process::CommandExt;
        match Command::new(&exe).args(&args).current_dir(&path).creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP | CREATE_NO_WINDOW).spawn() {
            Ok(c) => {
                Log::push_info(app.log, 211, Some(format!("{} {}. PID: {}", &exe, args.join(" "), c.id())));
            },
            Err(e) => {
                Log::push_stop(app.log, 212, Some(format!("{} {}. Error: {}", &exe, args.join(" "), e.to_string())));
            },
        };
    }

    #[cfg(not(target_family="windows"))]
    fn start(app: App) {
        let path = &app.init.exe_path;
        let exe = &app.init.exe_file;

        let args = vec!["go", "-r", &app.init.root_path];
        match Command::new(&exe).args(&args[..]).current_dir(&path).spawn() {
            Ok(c) => {
                Log::push_info(app.log, 211, Some(format!("{} {}. PID: {}", &exe, args.join(" "), c.id())));
            },
            Err(e) => {
                Log::push_stop(app.log, 212, Some(format!("{} {}. Error: {}", &exe, args.join(" "), e.to_string())));
            },
        };
    }

    #[cfg(target_family="windows")]
    fn to_win_path(text: &str) -> String {
        text.replace("/", "\\")
    }
}