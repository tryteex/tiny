use std::collections::HashMap;

use crate::work::{action::{Action, Answer, Data, Redirect}, html::Html};

pub fn breadcrumbs(this: &mut Action) -> Answer {
    if !this.internal {
        this.response.redirect = Some(Redirect { url: "/".to_owned(), permanently: true });
        return Answer::None;
    }

    this.data.insert("home", Data::String(this.lang("home")));
    Html::render("breadcrumbs", &this)
}

pub fn index(this: &mut Action) -> Answer {
    if !this.internal {
        this.response.redirect = Some(Redirect { url: "/".to_owned(), permanently: true });
        return Answer::None;
    }

    this.data.insert("title", Data::String(this.lang("title")));
    this.data.insert("description", Data::String(this.lang("description")));
    this.data.insert("lang", Data::String(this.language.langs[this.session.get_lang() as usize].lang.clone()));

    this.load("header", "index", "main", "header", None);
    this.load("footer", "index", "main", "footer", None);
    this.load("sidebar", "index", "main", "sidebar", None);

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
    this.load("navigation", "index", "main", "navigation", None);

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

    this.data.insert("language", Data::String(this.language.langs[this.session.get_lang() as usize].name.clone()));

    let mut langs = Vec::with_capacity(this.language.langs.len());
    for lang in &this.language.langs {
        let mut map = HashMap::with_capacity(2);
        map.insert("langs.id".to_owned(), Data::U64(lang.id));
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
    if this.request.ajax {
        return Answer::String("404".to_owned());
    }
    this.data.insert("home", Data::String(this.lang("home")));
    this.data.insert("not_found", Data::String(this.lang("not_found")));
    this.load("header", "index", "main", "header", None);
    this.load("footer", "index", "main", "footer", None);
    this.response.http_code = Some(404);
    Html::render("404", &this)
}