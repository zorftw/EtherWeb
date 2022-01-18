use rocket::http::Cookie;
use rocket::http::Cookies;
use rocket::response::Flash;
use rocket::response::Redirect;
use rocket::{Request, Outcome};
use rocket::request::FromRequest;
use std::collections::HashMap;
use std::str::{from_utf8};
use super::authorization::*;

use super::PGCONN;



/// Cookie that is used to indicate that a user is actually logged in.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisteredUserCookie {
    pub userid: u32,
    pub username: String,
    pub display: Option<String>,
    pub purchased: bool,
}

/// Used for handling requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisteredUserForm {
    pub username: String,
    pub password: String,
}

impl CookieId for RegisteredUserCookie {
    fn cookie_id<'a>() -> &'a str {
        "lebron_james"
    }
}

impl CookieId for RegisteredUserForm {
    fn cookie_id<'a>() -> &'a str {
        "lebron_james"
    }
}

impl AuthorizeCookie for RegisteredUserCookie {
    /// The store_cookie() method should contain code that
    /// converts the specified data structure into a string
    fn store_cookie(&self) -> String {
        ::serde_json::to_string(self).expect("Could not serialize cookie")
    }

    /// The retrieve_cookie() method deserializes a string
    /// into a cookie data type.
    #[allow(unused_variables)]
    fn retrieve_cookie(string: String) -> Option<Self> {
        let mut des_buf = string.clone();
        let des: Result<RegisteredUserCookie, _> = ::serde_json::from_str(&mut des_buf);
        if let Ok(cooky) = des {
            Some(cooky)
        } else {
            None
        }
    }
}

impl AuthorizeForm for RegisteredUserForm {
    type CookieType = RegisteredUserCookie;

    fn authenticate(&self) -> Result<RegisteredUserCookie, AuthFail> {
        let conn = PGCONN.lock().unwrap();
        let authstr = format!(r#"
            SELECT u.userid, u.username, u.display, u.purchased FROM users u WHERE u.username = '{username}' AND 
                u.salt_hash = crypt('{password}', u.salt_hash)"#, username=&self.username, password=&self.password);

        let is_user_qrystr = format!("SELECT userid FROM users WHERE username = '{}'", &self.username);
        let is_admin_qrystr = format!("SELECT userid FROM users WHERE username = '{}' AND purchased = '1'", &self.username);
        let password_qrystr = format!("SELECT userid FROM users WHERE username = '{}' AND salt_hash = crypt('{}', salt_hash)", &self.username, &self.password);
        if let Ok(qry) = conn.query(&authstr, &[]) {
            if !qry.is_empty() && qry.len() == 1 {
                let row = qry.get(0);
                
                let display_opt = row.get_opt(2);
                let display = match display_opt {
                    Some(Ok(d)) => Some(d),
                    _ => None,
                };
                return Ok(RegisteredUserCookie {
                    userid: row.get(0),
                    username: row.get(1),
                    display,
                    purchased: row.get(3),
                });
            }
        }
        if let Ok(eqry) = conn.query(&is_user_qrystr, &[]) {
            if eqry.is_empty() || eqry.len() == 0 {
                return Err(AuthFail::new(self.username.clone(), "Username was not found.".to_string()));
            }
        }
        if let Ok(eqry) = conn.query(&is_admin_qrystr, &[]) {
            if eqry.is_empty() || eqry.len() == 0 {
                // In production this message may be more harmful than useful as it
                // would be able to tell anyone who is an administrator and thus the
                // message should be changed to something like Unkown error or Invalid username/password
                return Err(AuthFail::new(self.username.clone(), "User does not have access to user pages.".to_string()));
            }
        }
        if let Ok(eqry) = conn.query(&password_qrystr, &[]) {
            if eqry.is_empty() || eqry.len() == 0 {
                return Err(AuthFail::new(self.username.clone(), "Invalid username / password combination.".to_string()));
            }
        }
        Err(AuthFail::new(self.username.clone(), "Unknown error..".to_string()))
    }

     /// Create a new login form instance
     fn new_form(user: &str, pass: &str, _extras: Option<HashMap<String, String>>) -> Self {
        RegisteredUserForm {
            username: user.to_string(),
            password: pass.to_string(),
        }
    }

     /// Define a custom flash_redirect() method that overrides the default
    /// implementation in authorization::AuthorizeForm trait.
    /// This allows the cookie to be made secure
    fn flash_redirect(&self, ok_redir: impl Into<String>, err_redir: impl Into<String>, cookies: &mut Cookies) -> Result<Redirect, Flash<Redirect>> {
        match self.authenticate() {
            Ok(cooky) => {
                let cid = Self::cookie_id();
                let contents = cooky.store_cookie();
                cookies.add_private(
                    Cookie::build(cid, contents)
                        // .secure(true)
                        .finish()
                );
                Ok(Redirect::to(ok_redir.into()))
            },
            Err(fail) => {
                let mut furl = err_redir.into();
                if &fail.user != "" {
                    let furl_qrystr = Self::fail_url(&fail.user);
                    furl.push_str(&furl_qrystr);
                }
                Err( Flash::error(Redirect::to(furl), &fail.msg) )
            },
        }
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for RegisteredUserCookie {
    type Error = ();
    
    /// The from_request inside the file defining the custom data types
    /// enables the type to be checked directly in a route as a request guard
    /// 
    /// This is not needed but highly recommended.  Otherwise you would need to use:
    /// 
    /// `#[get("/protected")] fn admin_page(admin: AuthCont<RegisteredUserCookie>)`
    /// 
    /// instead of:
    /// 
    /// `#[get("/protected")] fn admin_page(admin: RegisteredUserCookie)`
    fn from_request(request: &'a Request<'r>) -> ::rocket::request::Outcome<RegisteredUserCookie,Self::Error>{
        let cid = RegisteredUserCookie::cookie_id();
        let mut cookies = request.cookies();
        
        match cookies.get_private(cid) {
            Some(cookie) => {
                if let Some(cookie_deserialized) = RegisteredUserCookie::retrieve_cookie(cookie.value().to_string()) {
                    Outcome::Success(
                        cookie_deserialized
                    )
                } else {
                    Outcome::Forward(())
                }
            },
            None => Outcome::Forward(())
        }
    }
}

