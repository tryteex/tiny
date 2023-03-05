use crate::work::{action::{Action, Answer}};

pub fn index(this: &mut Action) -> Answer {
    this.load_raw("index", "main", "index", None)
}

pub fn not_found(this: &mut Action) -> Answer {
    this.load_raw("index", "main", "not_found", None)
}