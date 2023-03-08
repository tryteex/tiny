use std::{sync::{Mutex, Arc}, collections::HashMap, fs::{read_dir, read_to_string}};

use crate::sys::log::Log;

use super::action::{Answer, Data, Action};

#[derive(Debug, Clone)]
pub enum Node {
    Text(String),
    Value(String),
    ValueDeEscape(String),
    ValueEscape(String),
    If(String, Vec<Node>, Vec<Node>),
    Loop(String, Vec<Node>),
}

enum TypeNode<'a> {
    Value(&'a str),
    ValueDeEscape(&'a str),
    ValueEscape(&'a str),
    If(&'a str),
    Else,
    EndIf,
    Loop(&'a str),
    EndLoop,
    Comment,
    Err,
}

enum ConditionNode {
    End,
    EndIf(usize),
    ElseIf(usize),
    EndLoop(usize),
    Err,
}

#[derive(Debug, Clone)]
pub struct Html {
    list: HashMap<String, HashMap<String, HashMap<String, Vec<Node>>>>,
}

impl Html {
    
    // {% str %} - unescaped output
    // {%+ str %} - escaped output
    // {%- str %} - de-escaped output
    // {%# comment %} - comment
    // {%? bool %} - if
    // {%?~%} - else
    // {%?%} - end if
    // {%@ arr %} - loop vec
    // {%@%} - end loop
    fn get_view(html: &str, vec: &mut Vec<Node>, log: Arc<Mutex<Log>>) -> ConditionNode {
        if html.len() == 0 {
            return ConditionNode::End;
        }
        let mut ind = 0;
        loop {
            match html[ind..].find("{%") {
                Some(b) => {
                    if b != 0 {
                        vec.push(Node::Text(html[ind..ind+b].to_owned()))
                    }
                    match html[ind+b..].find("%}") {
                        Some(e) => {
                            let mut shift = 0;
                            match Html::get_type_node(&html[ind+b..ind+b+e+2], Arc::clone(&log)) {
                                TypeNode::Value(name) => vec.push(Node::Value(name.to_owned())),
                                TypeNode::ValueDeEscape(name) => vec.push(Node::ValueDeEscape(name.to_owned())),
                                TypeNode::ValueEscape(name) => vec.push(Node::ValueEscape(name.to_owned())),
                                TypeNode::If(name) => {
                                    let mut vt = Vec::new();
                                    let mut vf = Vec::new();
                                    match Html::get_view(&html[ind + b + e + 2..], &mut vt, Arc::clone(&log)) {
                                        ConditionNode::EndIf(i) => {
                                            vec.push(Node::If(name.to_owned(), vt, vf));
                                            shift = i;
                                        },
                                        ConditionNode::ElseIf(i) => {
                                            match Html::get_view(&html[ind + b + e + 2 + i ..], &mut vf, Arc::clone(&log)) {
                                                ConditionNode::EndIf(j) => {
                                                    vec.push(Node::If(name.to_owned(), vt, vf));
                                                    shift = i + j;
                                                },
                                                ConditionNode::Err => return ConditionNode::Err,
                                                _ => {
                                                    Log::push_stop(log, 1201, Some(html[ind..ind+b+e+2].to_owned()));
                                                    return ConditionNode::Err;
                                                }
                                            };
                                        },
                                        ConditionNode::Err => return ConditionNode::Err,
                                        _ => {
                                            Log::push_stop(log, 1201, Some(html[ind..ind+b+e+2].to_owned()));
                                            return ConditionNode::Err;
                                        }
                                    };
                                },
                                TypeNode::Else => return ConditionNode::ElseIf(ind + b + e + 2 + shift),
                                TypeNode::EndIf => return ConditionNode::EndIf(ind + b + e + 2 + shift),
                                TypeNode::Loop(name) => {
                                    let mut v = Vec::new();
                                    match Html::get_view(&html[ind + b + e + 2..], &mut v, Arc::clone(&log)) {
                                        ConditionNode::EndLoop(i) => {
                                            vec.push(Node::Loop(name.to_owned(), v));
                                            shift = i;
                                        },
                                        ConditionNode::Err => return ConditionNode::Err,
                                        _ => {
                                            Log::push_stop(log, 1202, Some(html[ind..ind+b+e+2].to_owned()));
                                            return ConditionNode::Err;
                                        }
                                    };
                                },
                                TypeNode::EndLoop => return ConditionNode::EndLoop(ind + b + e + 2 + shift),
                                TypeNode::Comment => {},
                                TypeNode::Err => return ConditionNode::Err,
                            };
                            ind += b + e + 2 + shift;
                        },
                        None => break,
                    };
                },
                None => break,
            }
        }
        if ind < html.len() {
            vec.push(Node::Text(html[ind..].to_owned()));
        }
        ConditionNode::End
    }

    fn get_type_node<'a>(text: &'a str, log: Arc<Mutex<Log>>) -> TypeNode<'a> {
        let len = text.len();
        if len == 4 {
            Log::push_stop(log, 1200, Some(text.to_owned()));
            return TypeNode::Err;
        };
        if len == 5 {
            match &text[2..3] {
                "?" => return TypeNode::EndIf,
                "@" => return TypeNode::EndLoop,
                _ => {
                    Log::push_stop(log, 1200, Some(text.to_owned()));
                    return TypeNode::Err;
                },
            };
        };
        if len == 6 {
            match &text[2..4] {
                "?~" => return TypeNode::Else,
                _ => {
                    Log::push_stop(log, 1200, Some(text.to_owned()));
                    return TypeNode::Err;
                },
            };
        };
        if &text[2..3] == " " && &text[len-3..len-2] == " " {
            return TypeNode::Value(&text[3..len-3]);
        };
        if &text[2..4] == "- " && &text[len-3..len-2] == " " {
            return TypeNode::ValueDeEscape(&text[4..len-3]);
        };
        if &text[2..4] == "+ " && &text[len-3..len-2] == " " {
            return TypeNode::ValueEscape(&text[4..len-3]);
        };
        if &text[2..4] == "# " && &text[len-3..len-2] == " " {
            return TypeNode::Comment;
        };
        if &text[2..4] == "? " && &text[len-3..len-2] == " " {
            return TypeNode::If(&text[4..len-3]);
        };
        if &text[2..4] == "@ " && &text[len-3..len-2] == " " {
            return TypeNode::Loop(&text[4..len-3]);
        };
        Log::push_stop(log, 1200, Some(text.to_owned()));
        return TypeNode::Err;
    }

    pub fn new(root: &str, log: Arc<Mutex<Log>>) -> Option<Html> {
        let path = format!("{}/app/", root);
        let list = match read_dir(path) {
            Ok(r) => {
                let mut list = HashMap::new();
                for entry in r {
                    if let Ok(e) = entry {
                        let path = e.path();
                        if path.is_dir() {
                            if let Some(m) = path.file_name() {
                                if let Some(module) = m.to_str() {
                                    let mut ls = HashMap::new();
                                    if let Ok(r) = read_dir(&path) {
                                        for entry in r {
                                            if let Ok(e) = entry {
                                                let path = e.path();
                                                if path.is_dir() {
                                                    if let Some(c) = path.file_name() {
                                                        if let Some(class) = c.to_str() {
                                                            let mut l = HashMap::new();
                                                            if let Ok(r) = read_dir(&path) {
                                                                for entry in r {
                                                                    if let Ok(e) = entry {
                                                                        let path = e.path();
                                                                        if path.is_file() {
                                                                            if let Some(v) = path.file_name() {
                                                                                if let Some(view) = v.to_str() {
                                                                                    if view.ends_with(".html") && view.len() > 5{
                                                                                        if let Ok(str) = read_to_string(&path) {
                                                                                            let view = &view[..view.len()-5];
                                                                                            let mut vec = Vec::new();
                                                                                            if let ConditionNode::End = Html::get_view(&str, &mut vec, Arc::clone(&log)) {
                                                                                                l.insert(view.to_owned(), vec);
                                                                                            } else {
                                                                                                return None;
                                                                                            }
                                                                                        }
                                                                                    }
                                                                                }
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                            l.shrink_to_fit();
                                                            ls.insert(class.to_owned(), l);
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    ls.shrink_to_fit();
                                    list.insert(module.to_owned(), ls);
                                }
                            }
                        }
                    }
                }
                list.shrink_to_fit();
                list
            },
            Err(e) => {
                Log::push_warning(log, 1100, Some(e.to_string()));
                HashMap::new()
            },
        };
        Some(Html { 
            list,
        })
    }

    fn get_data(view: &str, data: &HashMap<&str, Data>, html: &Vec<Node>, add: Option<&HashMap<String, Data>>) -> String {
        let mut render = Vec::new();
        for node in html {
            match node {
                Node::Text(t) => render.push(t.to_owned()),
                Node::Value(key) => if let Some(val) = data.get(key as &str) {
                    match val {
                        Data::U8(v) => render.push(v.to_string()),
                        Data::I64(v) => render.push(v.to_string()),
                        Data::U64(v) => render.push(v.to_string()),
                        Data::F64(v) => render.push(v.to_string()),
                        Data::String(v) => render.push(v.to_owned()),
                        _ => {},
                    }
                } else if let Some(a) = add {
                    if let Some(val) = a.get(key as &str) {
                        match val {
                            Data::U8(v) => render.push(v.to_string()),
                            Data::I64(v) => render.push(v.to_string()),
                            Data::U64(v) => render.push(v.to_string()),
                            Data::F64(v) => render.push(v.to_string()),
                            Data::String(v) => render.push(v.to_owned()),
                            _ => {},
                        }
                    }
                },
                Node::ValueDeEscape(key) => if let Some(val) = data.get(key as &str) {
                    match val {
                        Data::U8(v) => render.push(v.to_string()),
                        Data::I64(v) => render.push(v.to_string()),
                        Data::U64(v) => render.push(v.to_string()),
                        Data::F64(v) => render.push(v.to_string()),
                        Data::String(v) => render.push(Html::de_escape(v)),
                        _ => {},
                    }
                } else if let Some(a) = add {
                    if let Some(val) = a.get(key as &str) {
                        match val {
                            Data::U8(v) => render.push(v.to_string()),
                            Data::I64(v) => render.push(v.to_string()),
                            Data::U64(v) => render.push(v.to_string()),
                            Data::F64(v) => render.push(v.to_string()),
                            Data::String(v) => render.push(Html::de_escape(v)),
                            _ => {},
                        }
                    }
                },
                Node::ValueEscape(key) => if let Some(val) = data.get(key as &str) {
                    match val {
                        Data::U8(v) => render.push(v.to_string()),
                        Data::I64(v) => render.push(v.to_string()),
                        Data::U64(v) => render.push(v.to_string()),
                        Data::F64(v) => render.push(v.to_string()),
                        Data::String(v) => render.push(Html::escape(v)),
                        _ => {},
                    }
                } else if let Some(a) = add {
                    if let Some(val) = a.get(key as &str) {
                        match val {
                            Data::U8(v) => render.push(v.to_string()),
                            Data::I64(v) => render.push(v.to_string()),
                            Data::U64(v) => render.push(v.to_string()),
                            Data::F64(v) => render.push(v.to_string()),
                            Data::String(v) => render.push(Html::escape(v)),
                            _ => {},
                        }
                    }
                },
                Node::If(key, vec_true, vec_false) => if let Some(val) = data.get(key as &str) {
                    if let Data::Bool(b) = val {
                        let r = if *b && vec_true.len() > 0 {
                            Html::get_data(view, data, vec_true, add)
                        } else if !*b && vec_false.len() > 0 { 
                            Html::get_data(view, data, vec_false, add)
                        } else {
                            "".to_owned()
                        };  
                        render.push(r);
                    }
                },
                Node::Loop(key, vec) => if let Some(val) = data.get(key as &str) {
                    match val {
                        Data::Vec(v) => for i in v {
                            if let Data::Map(m) = i {
                                let r = Html::get_data(view, data, vec, Some(m));
                                render.push(r);
                            }
                        },
                        _ => {}
                    };
                },
            }
        }
        if render.len() > 0 {
            return render.join("");
        }
        "".to_owned()
    }

    pub fn render(view: &str, this: &Action) -> Answer {
        match this.html {
            Some(view_set) => match view_set.get(view) {
                Some(v) => Answer::String(Html::get_data(view, &this.data, v, None)),
                None => Answer::None,
            },
            None => Answer::None,
        }
    }

    fn de_escape(text: &str) -> String {
        return text.to_owned();
    }

    fn escape(text: &str) -> String {
        let t = text.as_bytes();
        let mut len = 0;

        for b in t {
            len += match b {
                b'&' => 5,
                b'"' | b'\'' => 6,
                b'<' | b'>' => 4,
                _ => 0,
            };
        }
        if len == 0 {
            return text.to_owned();
        }
        let mut new_text = String::with_capacity(text.len() + len);
        for c in text.chars() {
            match c {
                '&' => new_text.push_str("&amp;"),
                '"' => new_text.push_str("&quot;"),
                '\'' => new_text.push_str("&apos;"),
                '<' => new_text.push_str("&lt;"),
                '>' => new_text.push_str("&gt;"),
                _ => new_text.push(c),
            };
        }
        return new_text;
    }

    pub fn get<'a>(&self, module: &str, class: &str) -> Option<&HashMap<String, Vec<Node>>> {
        if let Some(c) = self.list.get(module) {
            if let Some(v) = c.get(class) {
                return Some(v);
            };
        };
        None
    }
}