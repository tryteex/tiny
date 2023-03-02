pub mod app {
    tinymod::addmod!();
}

pub mod work {
    pub mod action;
    pub mod cache;
    pub mod db;
    pub mod worker;
    pub mod html;
    pub mod lang;
}
pub mod sys {
    pub mod log; 
    pub mod init;
    pub mod go;
    pub mod fastcgi;
    pub mod app;
}
pub mod help;

use std::sync::Arc;

use sys::{log::Log, app::App};

fn main() {
    let log = Log::new();
    let app = match App::new(Arc::clone(&log)) {
        Some(a) => a,
        None => {
            Log::stop(log);
            return
        },
    };
    Log::push_info(Arc::clone(&log), 200, Some(format!("mode={:?}", app.get_mode())));

    App::run(app);

    Log::push_info(Arc::clone(&log), 201, None);

    Log::stop(log);
}