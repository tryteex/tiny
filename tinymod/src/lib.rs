extern crate proc_macro;

use std::{env, str::FromStr, collections::{HashMap, hash_map::Entry}, fs::{read_dir, read_to_string}};
use proc_macro::{TokenStream, Span};
use syn::Error;

#[proc_macro]
pub fn addfn(_: TokenStream) -> TokenStream {
    let dir = match env::var_os("CARGO_MANIFEST_DIR") {
        Some(d) => match d.to_str() {
            Some(s) => s.to_owned(),
            None => return error("CARGO_MANIFEST_DIR contains non-printable characters"),
        },
        None => return error("Can't fetch the environment variable CARGO_MANIFEST_DIR"),
    };
    let list = match load_files(&dir) {
        Ok(l) => l,
        Err(e) => return error(&e),
    };
    let mut vec = Vec::new();
    vec.push(format!("let mut app: HashMap<&'static str, HashMap<&'static str, HashMap<&'static str, Act>>> = HashMap::with_capacity({});", list.len()));
    for (key, v) in list {
        vec.push(format!("let mut {}: HashMap<&'static str, HashMap<&'static str, Act>> = HashMap::with_capacity({});", key, v.len()));
        for file in v {
            let func = get_func(&dir, &key, &file);
            vec.push(format!("let mut {}_{}: HashMap<&'static str, Act> = HashMap::with_capacity({});", key, file, func.len()));
            for f in func {
                vec.push(format!("{}_{}.insert(\"{}\", crate::app::{}::{}::{});", key, file, f, key, file, f));
            }
            vec.push(format!("{}.insert(\"{}\", {}_{});", key, file, key, file));
        }
        vec.push(format!("app.insert(\"{}\", {});", key, key));
    }
    vec.push("return app;".to_owned());

    TokenStream::from_str(&vec.join("\n")).unwrap()
}

fn get_func(dir: &str, key: &str, file: &str) -> Vec<String> {
    let mut vec = Vec::new();
    let file = format!("{}/src/app/{}/{}.rs", dir, key, file);
    if let Ok(str) = read_to_string(file) {
        let mut str = str.replace("(", " ( ").replace(")", " ) ").replace(":", " : ").replace("->", " -> ").replace("{", " { ");
        loop {
            if str.contains("  ") {
                str = str.replace("  ", " ");
                continue;
            }
            break;
        }
        let mut ind = 0;
        loop {
            match &str[ind..].find("pub fn ") {
                Some(i) => {
                    match &str[ind + i + 7..].find(" ( this : &mut Action ) -> Answer {") {
                        Some(j) => {
                            vec.push(str[ind + i + 7 .. ind + i + 7 + j].to_owned());
                            ind += i + j + 7;
                        },
                        None => break,
                    }
                },
                None => break,
            };
        }
    }
    vec.shrink_to_fit();
    vec
}

#[proc_macro]
pub fn addmod(_: TokenStream) -> TokenStream {
    let dir = match env::var_os("CARGO_MANIFEST_DIR") {
        Some(d) => match d.to_str() {
            Some(s) => s.to_owned(),
            None => return error("CARGO_MANIFEST_DIR contains non-printable characters"),
        },
        None => return error("Can't fetch the environment variable CARGO_MANIFEST_DIR"),
    };
    let list = match load_files(&dir) {
        Ok(l) => l,
        Err(e) => return error(&e),
    };
    let mut vec = Vec::new();
    for (key, v) in list {
        vec.push(format!("pub mod {} {{", check_name(key)));
        for f in v {
            vec.push(format!("    pub mod {};", check_name(f)));
        }
        vec.push("}".to_owned());
    }
    TokenStream::from_str(&vec.join("\n")).unwrap()
}

fn check_name(text: String) -> String {
    if text.contains("-") {
        return text.replace("-", "_");
    }
    text
}

fn load_files(dir: &str) -> Result<HashMap<String, Vec<String>>, String> {
    let src = format!("{}/src/app", dir);
    let mut list: HashMap<String, Vec<String>> = HashMap::new();
    match read_dir(&src) {
        Ok(dir) => for entry in dir {
            if let Ok(e) = entry {
                let path = e.path();
                if path.is_dir() {
                    if let Some(name) = path.file_name() {
                        if let Some(dir_name) = name.to_str() {
                            if let Ok(dir) = read_dir(format!("{}/{}", &src, dir_name)) {
                                for entry in dir {
                                    if let Ok(e) = entry {
                                        let path = e.path();
                                        if path.is_file() {
                                            if let Some(name) = path.file_name() {
                                                if let Some(file_name) = name.to_str() {
                                                    if file_name.len() > 3 && file_name.ends_with(".rs") {
                                                        let file_name = file_name[..file_name.len()-3].to_owned();
                                                        match list.entry(dir_name.to_owned()) {
                                                            Entry::Occupied(mut o) => {
                                                                let vec = o.get_mut();
                                                                vec.push(file_name);
                                                                vec.shrink_to_fit();
                                                            },
                                                            Entry::Vacant(v) => {
                                                                let mut vec = Vec::new();
                                                                vec.push(file_name);
                                                                v.insert(vec);
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
        },
        Err(e) => return Err(format!("{}. File name: {}", e.to_string(), src)),
    };
    list.shrink_to_fit();
    Ok(list)
}

fn error(text: &str) -> TokenStream {
    TokenStream::from(Error::new(Span::call_site().into(), text).to_compile_error())
}

