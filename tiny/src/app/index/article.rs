use std::collections::HashMap;

use crate::work::{action::{Action, Answer, Redirect, Data}, html::Html};

pub fn index(this: &mut Action) -> Answer {
    let article = match &this.param {
        Some(article) => match article as &str{
            "about" | "travel" | "article" | "contact" | "terms" | "policy" => article.clone(),
            _ => {
                this.response.redirect = Some(Redirect { url: "/".to_owned(), permanently: true });
                return Answer::None;
            },
        },
        None => {
            this.response.redirect = Some(Redirect { url: "/".to_owned(), permanently: true });
            return Answer::None;
        },
    };
    this.load("article", "index", "article", &article, None);
    this.load("header", "index", "main", "header", None);
    this.load("footer", "index", "main", "footer", None);

    Html::render("index", &this)
}

pub fn about(this: &mut Action) -> Answer {
    if !this.internal {
        this.response.redirect = Some(Redirect { url: "/".to_owned(), permanently: true });
        return Answer::None;
    }
    this.load("sidebar", "index", "main", "sidebar", None);

    let mut breadcrumbs = Vec::new();
    let mut map = HashMap::with_capacity(2);
    map.insert("breadcrumbs.url".to_owned(), Data::String(this.route("index", "article", "index", Some("about"), None)));
    map.insert("breadcrumbs.name".to_owned(), Data::String(this.lang("about")));
    breadcrumbs.push(Data::Map(map));
    this.data.insert("breadcrumbs", Data::Vec(breadcrumbs));
    this.load("breadcrumbs", "index", "main", "breadcrumbs", None);

    Html::render("about", &this)
}

pub fn travel(this: &mut Action) -> Answer {
    if !this.internal {
        this.response.redirect = Some(Redirect { url: "/".to_owned(), permanently: true });
        return Answer::None;
    }

    let mut breadcrumbs = Vec::new();
    let mut map = HashMap::with_capacity(2);
    map.insert("breadcrumbs.url".to_owned(), Data::String(this.route("index", "article", "index", Some("travel"), None)));
    map.insert("breadcrumbs.name".to_owned(), Data::String(this.lang("articles")));
    breadcrumbs.push(Data::Map(map));
    this.data.insert("breadcrumbs", Data::Vec(breadcrumbs));
    this.load("breadcrumbs", "index", "main", "breadcrumbs", None);

    this.load("sidebar", "index", "main", "sidebar", None);

    Html::render("travel", &this)
}

pub fn article(this: &mut Action) -> Answer {
    if !this.internal {
        this.response.redirect = Some(Redirect { url: "/".to_owned(), permanently: true });
        return Answer::None;
    }
    this.data.insert("article", Data::String(this.lang("article")));

    let mut breadcrumbs = Vec::new();
    let mut map = HashMap::with_capacity(2);
    map.insert("breadcrumbs.url".to_owned(), Data::String(this.route("index", "article", "index", Some("travel"), None)));
    map.insert("breadcrumbs.name".to_owned(), Data::String(this.lang("articles")));
    breadcrumbs.push(Data::Map(map));
    let mut map = HashMap::with_capacity(2);
    map.insert("breadcrumbs.url".to_owned(), Data::String(this.route("index", "article", "index", Some("article"), None)));
    map.insert("breadcrumbs.name".to_owned(), Data::String(this.lang("article")));
    breadcrumbs.push(Data::Map(map));
    this.data.insert("breadcrumbs", Data::Vec(breadcrumbs));
    this.load("breadcrumbs", "index", "main", "breadcrumbs", None);

    this.load("sidebar", "index", "main", "sidebar", None);

    Html::render("article", &this)
}

pub fn contact(this: &mut Action) -> Answer {
    if !this.internal {
        this.response.redirect = Some(Redirect { url: "/".to_owned(), permanently: true });
        return Answer::None;
    }

    let mut breadcrumbs = Vec::new();
    let mut map = HashMap::with_capacity(2);
    map.insert("breadcrumbs.url".to_owned(), Data::String(this.route("index", "article", "index", Some("contact"), None)));
    map.insert("breadcrumbs.name".to_owned(), Data::String(this.lang("contact")));
    breadcrumbs.push(Data::Map(map));
    this.data.insert("breadcrumbs", Data::Vec(breadcrumbs));

    Html::render("contact", &this)
}

pub fn terms(this: &mut Action) -> Answer {
    if !this.internal {
        this.response.redirect = Some(Redirect { url: "/".to_owned(), permanently: true });
        return Answer::None;
    }

    let mut breadcrumbs = Vec::new();
    let mut map = HashMap::with_capacity(2);
    map.insert("breadcrumbs.url".to_owned(), Data::String(this.route("index", "article", "index", Some("terms"), None)));
    map.insert("breadcrumbs.name".to_owned(), Data::String(this.lang("terms")));
    breadcrumbs.push(Data::Map(map));
    this.data.insert("breadcrumbs", Data::Vec(breadcrumbs));
    this.load("breadcrumbs", "index", "main", "breadcrumbs", None);

    Html::render("terms", &this)
}

pub fn policy(this: &mut Action) -> Answer {
    if !this.internal {
        this.response.redirect = Some(Redirect { url: "/".to_owned(), permanently: true });
        return Answer::None;
    }

    let mut breadcrumbs = Vec::new();
    let mut map = HashMap::with_capacity(2);
    map.insert("breadcrumbs.url".to_owned(), Data::String(this.route("index", "article", "index", Some("policy"), None)));
    map.insert("breadcrumbs.name".to_owned(), Data::String(this.lang("policy")));
    breadcrumbs.push(Data::Map(map));
    this.data.insert("breadcrumbs", Data::Vec(breadcrumbs));
    this.load("breadcrumbs", "index", "main", "breadcrumbs", None);

    Html::render("policy", &this)
}
