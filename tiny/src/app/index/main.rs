use std::collections::HashMap;

use crate::work::{action::{Action, Answer, Data, Redirect}, html::Html};

pub fn index(this: &mut Action) -> Answer {
    if !this.internal {
        this.response.redirect = Some(Redirect { url: "/".to_owned(), permanently: true });
        return Answer::None;
    }

    this.data.insert("title", Data::String(this.lang("title")));
    this.data.insert("description", Data::String(this.lang("description")));
    this.data.insert("lang", Data::String(this.language.langs[this.session.lang_id as usize].lang.clone()));

    if let Answer::String(str) = this.load("index", "main", "header", None) {
        this.data.insert("header", Data::String(str));
    }
    if let Answer::String(str) = this.load("index", "main", "footer", None) {
        this.data.insert("footer", Data::String(str));
    }
    if let Answer::String(str) = this.load("index", "main", "sidebar", None) {
        this.data.insert("sidebar", Data::String(str));
    }
    Html::render("index", &this)
}

pub fn sidebar(this: &mut Action) -> Answer {
    if !this.internal {
        this.response.redirect = Some(Redirect { url: "/".to_owned(), permanently: true });
        return Answer::None;
    }
    Html::render("sidebar", &this)
}

pub fn header(this: &mut Action) -> Answer {
    if !this.internal {
        this.response.redirect = Some(Redirect { url: "/".to_owned(), permanently: true });
        return Answer::None;
    }
    if let Answer::String(str) = this.load("index", "main", "navigation", None) {
        this.data.insert("navigation", Data::String(str));
    }

    Html::render("header", &this)
}

pub fn navigation(this: &mut Action) -> Answer {
    if !this.internal {
        this.response.redirect = Some(Redirect { url: "/".to_owned(), permanently: true });
        return Answer::None;
    }
    this.data.insert("about", Data::String(this.lang("about")));
    this.data.insert("cruises", Data::String(this.lang("cruises")));
    this.data.insert("articles", Data::String(this.lang("articles")));
    this.data.insert("travel", Data::String(this.lang("travel")));
    this.data.insert("lifestyle", Data::String(this.lang("lifestyle")));
    this.data.insert("contact", Data::String(this.lang("contact")));

    this.data.insert("language", Data::String(this.language.langs[this.session.lang_id as usize].name.clone()));

    let mut langs = Vec::with_capacity(this.language.langs.len());
    for lang in &this.language.langs {
        let mut map = HashMap::with_capacity(2);
        map.insert("langs.id".to_owned(), Data::U8(lang.id));
        map.insert("langs.name".to_owned(), Data::String(lang.name.clone()));
        langs.push(Data::Map(map));
    }
    this.data.insert("langs", Data::Vec(langs));

    Html::render("navigation", &this)
}

pub fn footer(this: &mut Action) -> Answer {
    if !this.internal {
        this.response.redirect = Some(Redirect { url: "/".to_owned(), permanently: true });
        return Answer::None;
    }
    Html::render("footer", &this)
}

pub fn not_found(this: &mut Action) -> Answer {
    if !this.internal {
        this.response.redirect = Some(Redirect { url: "/".to_owned(), permanently: true });
        return Answer::None;
    }
    Html::render("not_found", &this)
}

pub fn err(this: &mut Action) -> Answer {
    if !this.internal {
        this.response.redirect = Some(Redirect { url: "/".to_owned(), permanently: true });
        return Answer::None;
    }
    Html::render("err", &this)
}