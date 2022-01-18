#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_include_static_resources;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate serde;
#[allow(unused_imports)] #[macro_use] extern crate serde_json;
extern crate rocket_contrib;
extern crate chrono;
extern crate r2d2;
extern crate r2d2_postgres;
extern crate postgres;
extern crate htmlescape;
extern crate unic_ucd;

use std::sync::Mutex;
use rocket_include_static_resources::{StaticResponse};
use rocket_contrib::{templates::Template, serve::StaticFiles};

mod auth;
mod index;

lazy_static! {
    static ref PGCONN: Mutex<auth::db::DbConn> = Mutex::new(auth::db::DbConn(auth::db::init_pg_pool().get().expect("Could not connect to DB")));
}

#[get("/favicon.ico")]
fn favicon() -> StaticResponse {
    static_response!("favicon")
}

fn main()
{
    rocket::ignite()
    .attach(StaticResponse::fairing(|resources| {
        static_resources_initialize!(resources,
            "favicon", "assets/icon/favicon.ico",
        );
    }))
    .manage(auth::db::init_pg_pool())
    .mount("/css", StaticFiles::from("templates/css"))
    .mount("/assets/", StaticFiles::from("assets/"))
    .mount("/", routes![favicon])
    .mount("/", routes![auth::process_login, auth::logged_in, auth::logout])
    .mount("/", routes![index::index_page])
    .attach(Template::fairing())
    .launch();
}
