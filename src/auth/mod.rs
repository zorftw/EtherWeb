pub mod db;
pub mod account;
pub mod sanitization;
pub mod authorization;

use rocket::response::{NamedFile, Redirect, Flash};
use rocket::response::content::Html;
use rocket::request::{FlashMessage, Form};
use rocket::http::{Cookie, Cookies};
use super::PGCONN;

use authorization::*;
use account::*;

#[post("/login", data="<form>")]
pub fn process_login(form: Form<authorization::LoginCont<account::RegisteredUserForm>>, mut cookies: Cookies) -> Result<Redirect, Flash<Redirect>> {
    let inner = form.into_inner();
    let login = inner.form;

    println!("Attempted login with params: {:?}", login);

    login.flash_redirect("/", "/login", &mut cookies)
}

#[get("/login", rank = 1)]
pub fn logged_in(_user: AuthCont<account::RegisteredUserCookie>, conn: db::DbConn) -> String {
    let cookie = _user.cookie;
    let qrystr = format!("SELECT userid, username, display FROM users WHERE username = '{}'", cookie.username);
    let user_data_qry = conn.query(&qrystr, &[]);
    let output = match user_data_qry {
        Ok(qry) => {
            if !qry.is_empty() && qry.len() == 1 {
                let row = qry.get(0);
                
                // the display field is null so use get_opt to get a result, which unwraps to a string
                let display_opt = row.get_opt(2);
                let display = match display_opt {
                    Some(Ok(d)) => Some(d),
                    _ => None,
                };
                
                let user_results = RegisteredUserCookie {
                    userid: row.get(0),
                    username: row.get(1),
                    display: display,
                    purchased: row.get(3),
                };
                format!("Welcome. Your info is:<br>\nId: {}<br>\nUsername: {}<br>\nDisplay name: {}", 
                    user_results.userid, user_results.username, user_results.display.unwrap_or(String::from("no display name")))
            } else {
                String::from("Could not retrieve the user from the database.")
            }
        },
        Err(err) => String::from("Could not query the database."),
    };
    output
}

#[get("/logout")]
pub fn logout(admin: Option<account::RegisteredUserCookie>, mut cookies: Cookies) -> Result<Flash<Redirect>, Redirect> {
    if let Some(_) = admin {
        cookies.remove_private(Cookie::named(account::RegisteredUserCookie::cookie_id()));
        Ok(Flash::success(Redirect::to("/"), "Successfully logged out."))
    } else {
        Err(Redirect::to("/login"))
    }
}
