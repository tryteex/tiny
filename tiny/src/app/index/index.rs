use crate::work::{action::{Action, Answer}};

pub fn index(this: &mut Action) -> Answer {
    this.load("index", "main", "index", None)
}

pub fn not_found(this: &mut Action) -> Answer {
    this.load("index", "main", "not_found", None)
}

pub fn err(this: &mut Action) -> Answer {
    this.response.http_code = Some(500);
    Answer::String("500".to_owned())
}