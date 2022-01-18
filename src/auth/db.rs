use core::ops::Deref;
use rocket::{
    request::{self, FromRequest, FromForm, FormItems},
    Request, State, Outcome,
    config::{Config, Environment},
    http::Status
};

use r2d2;
use r2d2_postgres;
use r2d2_postgres::{PostgresConnectionManager, TlsMode};
use postgres::Connection;
use postgres;
use postgres::params::{ConnectParams, Host};

type Pool = r2d2::Pool<PostgresConnectionManager>;

pub fn init_pg_pool() -> Pool {
    let cfg = r2d2::Config::default();
    let manager = PostgresConnectionManager::new("postgres://postgres@localhost/login", TlsMode::None).unwrap();

    r2d2::Pool::new(cfg, manager).expect("Could not create a database pool")
}
/// DbConn is a data structure that contains a database connection.
/// The DbConn also has a request guard that retrieves a connection
/// from the shared state when the route is called.
pub struct DbConn(
    pub r2d2::PooledConnection<PostgresConnectionManager>
);

/// Attempts to retrieve a single connection from the managed database pool. If
/// no pool is currently managed, fails with an `InternalServerError` status. If
/// no connections are available, fails with a `ServiceUnavailable` status.
impl<'a, 'r> FromRequest<'a, 'r> for DbConn {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<DbConn, ()> {
        let pool = request.guard::<State<Pool>>()?;
        match pool.get() {
            Ok(conn) => Outcome::Success(DbConn(conn)),
            Err(_) => Outcome::Failure((Status::ServiceUnavailable, ()))
        }
    }
}

// For the convenience of using an &DbConn as a postgresql connection.
impl Deref for DbConn {
    // If using Sqlite
    // type Target = SqliteConnection;
    type Target = Connection;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}