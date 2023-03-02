pub struct Help;

impl Help {
    pub fn show() {
        let desc = "Tiny is a high-speed FastCGI server for WEB applications.";
        let ver = format!("tiny version: {}", env!("CARGO_PKG_VERSION"));
        let help = "
    Usage: tiny [start|stop|help] [-r <path to root path>]
    
    Actions:
        start         : start server
        stop          : stop server
        help          : show this help
        
    ";
        println!("");
        println!("{}", desc);
        println!("{}", ver);
        println!("{}", help);
    }
}