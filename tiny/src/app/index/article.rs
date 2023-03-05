use crate::work::{action::{Action, Answer, Redirect}, html::Html};

pub fn index(this: &mut Action) -> Answer {
    let article = match &this.param {
        Some(article) => match article as &str{
            "about" | "travel" | "article" | "contact" => article.clone(),
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

    Html::render("about", &this)
}

pub fn travel(this: &mut Action) -> Answer {
    if !this.internal {
        this.response.redirect = Some(Redirect { url: "/".to_owned(), permanently: true });
        return Answer::None;
    }
    this.load("sidebar", "index", "main", "sidebar", None);

    Html::render("travel", &this)
}

pub fn article(this: &mut Action) -> Answer {
    if !this.internal {
        this.response.redirect = Some(Redirect { url: "/".to_owned(), permanently: true });
        return Answer::None;
    }
    this.load("sidebar", "index", "main", "sidebar", None);

    Html::render("article", &this)
}

pub fn contact(this: &mut Action) -> Answer {
    if !this.internal {
        this.response.redirect = Some(Redirect { url: "/".to_owned(), permanently: true });
        return Answer::None;
    }
    Html::render("contact", &this)
}
