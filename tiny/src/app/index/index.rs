use crate::work::{action::{Action, Answer}};

pub fn lang(this: &mut Action) -> Answer {
    if !this.internal && !this.request.ajax {
        return this.load_raw("index", "index", "not_found", None);
    }
    if let Some(p) = &this.param {
        if let Ok(id) = p.parse::<u64>() {
            if this.language.check(id) {
                this.session.set_lang(id);
                return Answer::String("ok".to_string());
            }
        }
    }
    this.load_raw("index", "index", "not_found", None)
}

pub fn index(this: &mut Action) -> Answer {
    this.load_raw("index", "main", "index", None)
}

pub fn not_found(this: &mut Action) -> Answer {
    this.load_raw("index", "main", "not_found", None)
}