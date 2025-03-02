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
use std::sync::atomic::{AtomicU64, Ordering};
use std::collections::HashMap;

#[derive(Clone)]
struct AppState {
    db_conn: DbConn,
    active_sessions: Arc<Mutex<HashMap<String, bool>>>,
    session_counter: Arc<AtomicU64>,
}

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

// Modify your login function to use a session token instead of username
#[post("/login", data = "<credentials>")]
async fn process_login(
    credentials: Form<Credentials>,
    cookies: &CookieJar<'_>,
    state: &State<AppState>,
) -> Result<Redirect, &'static str> {
    match state.db_conn.verify_password(&credentials.username, &credentials.password).await {
        Ok(true) => {
            // Generate a unique session token
            let session_id = format!("session_{}", state.session_counter.fetch_add(1, Ordering::SeqCst));
            
            // Store the session in our active sessions
            let mut sessions = state.active_sessions.lock().await;
            sessions.insert(session_id.clone(), true);
            
            // Set cookie with the session token
            let mut cookie = Cookie::new("session_id", session_id);
            cookie.set_path("/");
            cookie.set_http_only(true); // More secure
            cookies.add(cookie);
            
            Ok(Redirect::to("/dashboard.html"))
        }
        Ok(false) => Err("Invalid username or password"),
        Err(_) => Err("Database error"),
    }
}

#[get("/dashboard.html")]
async fn dash(
    cookies: &CookieJar<'_>,
    state: &State<AppState>,
) -> Result<(rocket::http::ContentType, Option<Vec<u8>>), Redirect> {
    if let Some(cookie) = cookies.get("session_id") {
        let session_id = cookie.value();
        let sessions = state.active_sessions.lock().await;
        
        if sessions.contains_key(session_id) {
            return Ok((
                rocket::http::ContentType::HTML,
                std::fs::read(relative!("templates/dashboard.html")).ok(),
            ));
        }
    }
    
    Err(Redirect::to("/login.html"))
}

#[get("/api.html")]
async fn api(
    cookies: &CookieJar<'_>,
    state: &State<AppState>,
) -> Result<(rocket::http::ContentType, Option<Vec<u8>>), Redirect> {
    if let Some(cookie) = cookies.get("session_id") {
        let session_id = cookie.value();
        let sessions = state.active_sessions.lock().await;
        
        if sessions.contains_key(session_id) {
            return Ok((
                rocket::http::ContentType::HTML,
                std::fs::read(relative!("templates/api.html")).ok(),
            ));
        }
    }
    
    Err(Redirect::to("/login.html"))
}

// Modify your logout function
#[get("/logout")]
async fn logout(
    cookies: &CookieJar<'_>,
    state: &State<AppState>,
) -> Redirect {
    // Check if session exists
    if let Some(cookie) = cookies.get("session_id") {
        let session_id = cookie.value();
        
        // Remove session from active sessions
        let mut sessions = state.active_sessions.lock().await;
        sessions.remove(session_id);
        
        // Remove cookie
        cookies.remove(Cookie::build("session_id"));
    }
    
    Redirect::to("/login.html")
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
    
    let app_state = AppState {
        db_conn,
        active_sessions: Arc::new(Mutex::new(HashMap::new())),
        session_counter: Arc::new(AtomicU64::new(1)),
    };

    rocket::build()
        .manage(app_state)
        .mount("/", routes![index, login, process_login, dash, api, favicon, logout])
        .mount("/", FileServer::from(relative!("templates")))
}