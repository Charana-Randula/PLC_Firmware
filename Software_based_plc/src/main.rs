#[macro_use]
extern crate rocket;

use rocket::tokio::sync::Mutex; // Import Mutex for async locking
use rocket::fs::{FileServer, relative}; // For serving static files
use rocket::response::Redirect; // For redirecting responses
use rocket::form::Form; // For handling forms
use rocket::http::{Cookie, CookieJar}; // For handling cookies
use rocket::State; // For managing application state

use rusqlite::{Connection, params, Result}; // For interacting with SQLite
use bcrypt::verify; // For bcrypt password hashing
use std::sync::Arc; // For wrapping Mutex in Arc to share across threads

#[derive(FromForm)]
struct Credentials {
    username: String,
    password: String,
}

#[derive(Clone)]
struct User {
    password: String,
}

#[derive(Clone)]
struct DbConn {
    conn: Arc<Mutex<Connection>>,
}

impl DbConn {
    fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        Ok(DbConn {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    // Make get_user async
    async fn get_user(&self, username: &str) -> Result<Option<User>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare("SELECT id, username, password FROM users WHERE username = ?")?;
        let mut user_iter = stmt.query_map(params![username], |row| {
            Ok(User {
                password: row.get(2)?,
            })
        })?;

        if let Some(user) = user_iter.next() {
            user.map(Some)
        } else {
            Ok(None)
        }
    }

    // Make verify_password async
    async fn verify_password(&self, username: &str, password: &str) -> Result<bool> {
        if let Some(user) = self.get_user(username).await? {
            Ok(verify(password, &user.password).unwrap_or(false))
        } else {
            Ok(false)
        }
    }
}

// Redirect root to login.html
#[get("/")]
fn index() -> Redirect {
    Redirect::to("/login.html")
}

// Serve login.html page
#[get("/login.html")]
fn login() -> (rocket::http::ContentType, Option<Vec<u8>>) {
    (
        rocket::http::ContentType::HTML,
        std::fs::read(relative!("templates/login.html")).ok(),
    )
}

#[post("/login", data = "<credentials>")]
async fn process_login(
    credentials: Form<Credentials>,
    cookies: &CookieJar<'_>,
    db_conn: &State<DbConn>,
) -> Result<Redirect, &'static str> {
    match db_conn.verify_password(&credentials.username, &credentials.password).await {
        Ok(true) => {
            cookies.add(Cookie::new("user_id", credentials.username.clone())); // Set cookie for session
            Ok(Redirect::to("/dashboard.html"))
        }
        Ok(false) => Err("Invalid username or password"),
        Err(_) => Err("Database error"),
    }
}

// Dashboard route that requires authentication
#[get("/dashboard.html")]
fn dash(cookies: &CookieJar<'_>) -> Result<(rocket::http::ContentType, Option<Vec<u8>>), Redirect> {
    if cookies.get("user_id").is_some() {
        Ok((
            rocket::http::ContentType::HTML,
            std::fs::read(relative!("templates/dashboard.html")).ok(),
        ))
    } else {
        Err(Redirect::to("/login.html")) // Redirect to login if not logged in
    }
}

// Dashboard route that requires authentication
#[get("/api.html")]
fn api(cookies: &CookieJar<'_>) -> Result<(rocket::http::ContentType, Option<Vec<u8>>), Redirect> {
    if cookies.get("user_id").is_some() {
        Ok((
            rocket::http::ContentType::HTML,
            std::fs::read(relative!("templates/api.html")).ok(),
        ))
    } else {
        Err(Redirect::to("/login.html")) // Redirect to login if not logged in
    }
}

// Handle missing favicon.ico requests gracefully
#[get("/favicon.ico")]
fn favicon() -> &'static str {
    ""
}

// Launch the Rocket app
#[launch]
fn rocket() -> _ {
    let db_conn = DbConn::new("db/users.db").expect("Failed to initialize database connection");

    rocket::build()
        .manage(db_conn) // Share the database connection globally
        .mount("/", routes![index, login, process_login, dash,api, favicon])
        .mount("/", FileServer::from(relative!("templates")))
}