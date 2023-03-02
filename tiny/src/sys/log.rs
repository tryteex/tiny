use std::{thread::{JoinHandle, self}, sync::{Mutex, Arc, mpsc}, path::PathBuf, process, fs::OpenOptions, io::Write};

use chrono::Local;

enum LogEvent {
    Exit,
    Add(LogText),
}

#[derive(Debug)]
enum LogView{
    Info,       // Informational message only
    Warning,    // Warning, the program may continue to run
    Stop,       // Error, the program must soft stop
    Error,      // Abnormal behavior, the program stops immediately.
    Critical,   // Critical error, the program stops immediately.
}

#[derive(Debug)]
pub struct LogText {
    view: LogView,
    number: u16,
    text: Option<String>,
}

#[derive(Debug)]
pub struct Log {
    file: String,
    thread: Option<JoinHandle<()>>,
    pid: u32,
    sender: Arc<Mutex<mpsc::Sender<LogEvent>>>,
}

impl Log {
    pub fn new() -> Arc<Mutex<Log>> {
        let (sender, receiver) = mpsc::channel();

        let log = Arc::new(Mutex::new(Log {
            file: "tiny.log".to_owned(),
            thread: None,
            pid: process::id(),
            sender: Arc::new(Mutex::new(sender)),
        }));
        
        let log_thread = Arc::clone(&log);
        let thread = thread::spawn(move || {
            loop {
                match receiver.recv() {
                    Ok(event) => match event {
                        LogEvent::Exit => break,
                        LogEvent::Add(t) => {
                            match Mutex::lock(&log_thread) {
                                Ok(l) => l.save(t),
                                Err(e) => Log::panic(e.to_string()),
                            }
                        },
                    },
                    Err(_) => break,
                }
            }
        });
        match Mutex::lock(&log) {
            Ok(mut l) => l.thread = Some(thread),
            Err(e) => Log::panic(e.to_string()),
        }
        log
    }

    fn save(&self, log: LogText) {
        let time = Local::now().format("%Y.%m.%d %H:%M:%S%.9f").to_string();
        let str = format!("ID:{} {} {:?}: {}\n", self.pid, time, log.view, Log::get_description(log.number, log.text));
        match OpenOptions::new().create(true).write(true).append(true).open(&self.file) {
            Ok(mut file) => match file.write_all(str.as_bytes()) {
                Ok(f) => f,
                Err(e) => Log::panic(e.to_string()),
            },
            Err(e) => {
                let str = format!("ID:{} {} {:?}: Can't open log file \"{}\" - {}\n", self.pid, time, LogView::Critical, &self.file, e.to_string());
                eprint!("{}", &str);
            },
        };
    }

    pub fn stop(log: Arc<Mutex<Log>>) {
        let sender = match Mutex::lock(&log) {
            Ok(l) => Arc::clone(&l.sender),
            Err(e) => Log::panic(e.to_string()),
        };
        match Mutex::lock(&sender) {
            Ok(s) => if let Err(e) = s.send(LogEvent::Exit) {
                Log::error(Arc::clone(&log), e.to_string());
            },
            Err(e) => Log::error(Arc::clone(&log), e.to_string()),
        }
        let thread = match Mutex::lock(&log) {
            Ok(mut l) => l.thread.take(),
            Err(e) => Log::panic(e.to_string()),
        };

        if let Some(thread) = thread {
            if let Err(_) = thread.join() {
                Log::error(Arc::clone(&log), "Couldn't join on the associated thread in log".to_string());
            };
        }
    }

    /// Informational message only
    pub fn push_info(log: Arc<Mutex<Log>>, number: u16, text: Option<String>) {
        Log::push(log, LogView::Info, number, text);
    }

    /// Warning, the program may continue to run
    pub fn push_warning(log: Arc<Mutex<Log>>, number: u16, text: Option<String>) {
        Log::push(log, LogView::Warning, number, text);
    }

    /// Error, the program must soft stop
    pub fn push_stop(log: Arc<Mutex<Log>>, number: u16, text: Option<String>) {
        Log::push(log, LogView::Stop, number, text);
    }

    /// Abnormal behavior, the program stops immediately.
    pub fn push_error(log: Arc<Mutex<Log>>, number: u16, text: Option<String>) -> ! {
        match Mutex::lock(&log) {
            Ok(g) => {
                g.save( LogText { view: LogView::Error, number, text});
                process::exit(1);
            },
            Err(e) => Log::panic(e.to_string()),
        }
    }

    // Critical error in the program. It is used when Mutex::lock() function cannot be called. The program stops immediately.
    pub fn error(log: Arc<Mutex<Log>>, text: String) -> ! {
        Log::push_error(log, 1, Some(text))
    }

    fn push(log: Arc<Mutex<Log>>, view: LogView, number: u16, text: Option<String>) {
        match Mutex::lock(&log) {
            Ok(mut l) => l.push_str(view, number, text),
            Err(e) => {
                Log::panic(e.to_string());
            },
        }
    }

    fn push_str(&mut self, view: LogView, number: u16, text: Option<String>) {
        match Mutex::lock(&self.sender) {
            Ok(s) => if let Err(e) = s.send(LogEvent::Add(LogText {view, number, text, })) {
                Log::panic(e.to_string())
            },
            Err(e) => Log::panic(e.to_string()),
        }
    }

    fn get_description(number: u16, text: Option<String>) -> String {
        let data = Log::number_to_text(number);
        match text {
            Some(text) => format!("{} => {}: {}", number, data, text),
            None => format!("{} => {}", number, data),
        }
    }

    pub fn set_path(log: Arc<Mutex<Log>>, path: String) {
        match Mutex::lock(&log) {
            Ok(mut g) => {
                g.file = path;
            },
            Err(e) => Log::panic(e.to_string()),
        }
    }

    fn panic(text: String) -> ! {
        let time = Local::now().format("%Y.%m.%d %H:%M:%S%.9f").to_string();
        let str = format!("{} {:?}: {}\n", time, LogView::Critical, Log::get_description(0, Some(text)));
        let file = PathBuf::from("tiny.log");
        match OpenOptions::new().create(true).write(true).append(true).open(&file) {
            Ok(mut f) => if let Err(e) = f.write_all(str.as_bytes()) {
                let str = format!("{} {:?}: Can't write log file \"{}\" - {}\n", time, LogView::Critical, file.display(), e.to_string());
                eprint!("{}", &str);
            },
            Err(e) => {
                let str = format!("{} {:?}: Can't open log file \"{}\" - {}\n", time, LogView::Critical, file.display(), e.to_string());
                eprint!("{}", &str);
            },
        };
        process::exit(1);
    }

    fn number_to_text(number: u16) -> &'static str {
        match number {
            0 => "Panic error",
            1 => "Critical error, Mutex::lock() not working",

            10 => "Unable to get the app path",
            11 => "The app path contains invalid characters",
            12 => "The app must be on a local computer",
            13 => "There is no path to the config file specified after the -r option",
            14 => "Can't read the config file",
            15 => "The config file is not found",
            16 => "Can't detect the app path",

            50 => "Error parsing the config file",
            51 => "The option \"log\" in the config file is required",
            52 => "The option \"log\" in the config file must be a string",
            53 => "The option \"log\" in the config file must be a path to file",
            54 => "The option \"version\" in the config file is required",
            55 => "The option \"version\" in the config file must be a string",
            56 => "The option \"max\" in the config file is required",
            57 => "The option \"max\" in the config file must be a number",
            58 => "The option \"max\" in the config file must be a u8",
            59 => "The option \"port\" in the config file is required",
            60 => "The option \"port\" in the config file must be a number",
            61 => "The option \"port\" in the config file must be a u16",
            62 => "The option \"ip\" in the config file is required",
            63 => "The option \"ip\" in the config file must be a string",
            64 => "The option \"ip\" in the config file must be a IP adress",
            65 => "The option \"rpc_port\" in the config file is required",
            66 => "The option \"rpc_port\" in the config file must be a number",
            67 => "The option \"rpc_port\" in the config file must be a u16",
            68 => "The option \"rpc_ip\" in the config file is required",
            69 => "The option \"rpc_ip\" in the config file must be a string",
            70 => "The option \"rpc_ip\" in the config file must be a IP adress",
            71 => "The option \"zone\" in the config file is required",
            72 => "The option \"zone\" in the config file must be a string",
            73 => "The option \"salt\" in the config file is required",
            74 => "The option \"salt\" in the config file must be a string",
            75 => "The option \"db\" in the config file is required",
            76 => "The option \"db\" in the config file must be a object",
            77 => "The option \"host\" in the object \"db\" in the config file is required",
            78 => "The option \"host\" in the object \"db\" in the config file must be a string",
            79 => "The option \"port\" in the object \"db\" in the config file is required",
            80 => "The option \"port\" in the object \"db\" in the config file must be a number",
            81 => "The option \"port\" in the object \"db\" in the config file must be a u16",
            82 => "The option \"name\" in the object \"db\" in the config file is required",
            83 => "The option \"name\" in the object \"db\" in the config file must be a string",
            84 => "The option \"user\" in the object \"db\" in the config file is required",
            85 => "The option \"user\" in the object \"db\" in the config file must be a string",
            86 => "The option \"pwd\" in the object \"db\" in the config file is required",
            87 => "The option \"pwd\" in the object \"db\" in the config file must be a string",
            88 => "The option \"rpc_accept\" in the config file is required",
            89 => "The option \"rpc_accept\" in the config file must be a number",
            90 => "The option \"rpc_accept\" in the config file must be a u16",
            91 => "The option \"accept\" in the config file must be a u16",
            92 => "The option \"accept\" in the config file must be a u16",
            93 => "The option \"accept\" in the config file must be a u16",
            94 => "The option \"lang\" in the config file is required",
            95 => "The option \"lang\" in the config file must be a number",
            96 => "The option \"lang\" in the config file must be a u8",

            200 => "Start",
            201 => "Stop",
            202 => "Unable to open rpc port",
            203 => "Unable to establish connection from rpc client",
            204 => "Unable to set read timeout on rpc port",
            205 => "Unable to read data on rpc port",
            206 => "An invalid command was received on the rpc port",
            207 => "Stop command was received on the rpc port",
            208 => "Can't detect the remove address",
            209 => "Accept remove connection",
            210 => "Joining from an illegal IP address",
            211 => "The app start succesful",
            212 => "Can't start the app",
            213 => "Can't connect to the server",
            214 => "Can't send 'stop' signal to the server",
            215 => "'Stop' signal sent successfully",
            216 => "Can't write 'stop' signal to the stream",
            217 => "Can't set read_timeout",
            218 => "Can't read signal from stream",
            219 => "Read data is very short",
            220 => "Read wrong data",

            500 => "Unable to open fastcgi server port",
            501 => "Can't join main process",
            502 => "Can't receive incomming request",
            503 => "Can't detect the remove address",
            504 => "Joining from an illegal IP address",
            505 => "Can't peek data from stream",
            506 => "Can't connect to the server",
            507 => "Can't send 'stop' signal to the server",
            508 => "Can't shutdown the stop socket",
            509 => "Can't join run worker process",
            510 => "Can't send run signal into worker process",
            511 => "Can't receive run signal",
            512 => "Can't send terminate signal",

            600 => "Can't create tlsconnector to database",
            601 => "Can't connect to database",
            602 => "Can't execute query",
            603 => "Can't init database",
            604 => "Can't prepare statement #1",

            700 => "Error in mpsc::Receiver",
            701 => "Can't send stop signal to the workers",
            702 => "Can't join worker process",
            703 => "Can't send ready signal from the workers",
            704 => "Threads are inconsistent",
            705 => "Can't send work signal to the workers",
            706 => "Sender is disconected",

            1020 => "Can't delete input file",

            1100 => "Can't open root_dir/app",

            1150 => "Can't load languages from database",
            1151 => "Language list is empty",
            1152 => "lang_id must be > 0",
            1153 => "Can't open root_dir/app",

            1200 => "Unable to specify node type",
            1201 => "Unable to specify \"if\" node type",
            1202 => "Unable to specify \"loop\" node type",

            _ => "Unknown error"
        }
    }
}
