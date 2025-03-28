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
use time::{Duration, OffsetDateTime};
use serde::Serialize;
use sysinfo::{NetworkExt, NetworksExt, ProcessorExt, System, SystemExt};

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
fn login(cookies: &CookieJar<'_>) -> (rocket::http::ContentType, Option<Vec<u8>>) {
    // Remove all cookies with path "/"
    for cookie in cookies.iter() {
        let mut expired_cookie = Cookie::from(cookie.name().to_string());
        expired_cookie.set_path("/"); // Ensure correct path
        expired_cookie.set_expires(OffsetDateTime::UNIX_EPOCH); // Expire immediately
        cookies.remove(expired_cookie); // Properly remove the cookie
    }

    // Serve the login page
    (
        rocket::http::ContentType::HTML,
        std::fs::read(relative!("templates/login.html")).ok(),
    )
}

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

            let mut now = OffsetDateTime::now_utc();
            now += Duration::hours(2);
            cookie.set_expires(now);

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

// Handle missing favicon.ico requests gracefully
#[get("/favicon.ico")]
fn favicon() -> &'static str {
    ""
}

#[derive(Serialize)]
struct SystemStats {
    cpu_usage: Vec<f32>,
    packets_in_sec: u64,
    packets_out_sec: u64,
    memory_used: u64,
    memory_total: u64,
    memory_pressure: f32,
}

#[get("/stats")]
async fn stats(
    cookies: &CookieJar<'_>,
    state: &State<AppState>,
) -> Result<rocket::serde::json::Json<SystemStats>, Redirect> {
    // First check if user is authenticated
    if let Some(cookie) = cookies.get("session_id") {
        let session_id = cookie.value();
        let sessions = state.active_sessions.lock().await;
        
        if sessions.contains_key(session_id) {
            let mut system = System::new_all();
            system.refresh_all();

            // Collect CPU usage stats
            let cpu_usage: Vec<f32> = system
                .processors()
                .iter()
                .map(|cpu| cpu.cpu_usage())
                .collect();

            // Get network stats
            let (packets_in_sec, packets_out_sec) = system.networks().iter().fold((0, 0), |acc, (_, data)| {
                (
                    acc.0 + (data.received()),
                    acc.1 + (data.transmitted()),
                )
            });

            // Get memory stats
            let memory_used = system.used_memory();
            let memory_total = system.total_memory();
            let memory_pressure = (memory_used as f32 / memory_total as f32) * 100.0;
            
            return Ok(rocket::serde::json::Json(SystemStats {
                cpu_usage,
                packets_in_sec,
                packets_out_sec,
                memory_used,
                memory_total,
                memory_pressure,
            }));
        }
    }
    
    Err(Redirect::to("/login.html"))
}

// Add this new route handler
#[get("/stats.html")]
async fn stats_page(
    cookies: &CookieJar<'_>,
    state: &State<AppState>,
) -> Result<(rocket::http::ContentType, Option<Vec<u8>>), Redirect> {
    if let Some(cookie) = cookies.get("session_id") {
        let session_id = cookie.value();
        let sessions = state.active_sessions.lock().await;
        
        if sessions.contains_key(session_id) {
            return Ok((
                rocket::http::ContentType::HTML,
                std::fs::read(relative!("templates/stats.html")).ok(),
            ));
        }
    }
    
    Err(Redirect::to("/login.html"))
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
        .mount("/", routes![index, login, process_login, dash, api, favicon, stats, stats_page])
        .mount("/", FileServer::from(relative!("templates")))
}