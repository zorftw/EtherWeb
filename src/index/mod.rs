extern crate rocket;

use rocket::request::FlashMessage;
use rocket_contrib::templates::Template;
use std::{collections::HashMap, time::SystemTime};
use chrono::{offset::Utc, DateTime};

use crate::auth::*;

#[get("/")]
pub fn index_page(_user: Option<account::RegisteredUserCookie>, _flash: Option<FlashMessage>, conn: db::DbConn) -> Template {
    let mut context = HashMap::new();
    let date_time: DateTime<Utc> = SystemTime::now().into();
    context.insert("time", format!("{}", date_time.format("%T")));
    
    let mut purchased_ctx = String::from("false");

    if let Some(user) = _user {
        purchased_ctx = format!("{}", if user.purchased {
            String::from("true")
        } else {
            String::from("false")
        });
    }

    context.insert("purchased", purchased_ctx);

    Template::render("index", context)
}