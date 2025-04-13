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
use rocket::serde::json::Json;
use rocket::request::{FromRequest, Outcome};
use rocket::Request;

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
    users_conn: Arc<Mutex<Connection>>,
    iov_conn: Arc<Mutex<Connection>>,
}

impl DbConn {
    fn new(users_db_path: &str, iov_db_path: &str) -> Result<Self> {
        let users_conn = Connection::open(users_db_path)?;
        let iov_conn = Connection::open(iov_db_path)?;
        Ok(DbConn {
            users_conn: Arc::new(Mutex::new(users_conn)),
            iov_conn: Arc::new(Mutex::new(iov_conn)),
        })
    }

    // User authentication methods use users_conn
    async fn get_user(&self, username: &str) -> Result<Option<User>> {
        let conn = self.users_conn.lock().await;
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

    async fn verify_password(&self, username: &str, password: &str) -> Result<bool> {
        if let Some(user) = self.get_user(username).await? {
            Ok(verify(password, &user.password).unwrap_or(false))
        } else {
            Ok(false)
        }
    }

    // IO and Variables methods use iov_conn
    async fn get_analog_inputs_outputs(&self, latest: bool) -> Result<Vec<AnalogInputsOutputs>> {
        let conn = self.iov_conn.lock().await;
        let query = if latest {
            "SELECT * FROM analog_inputs_outputs ORDER BY timestamp DESC LIMIT 1"
        } else {
            "SELECT * FROM analog_inputs_outputs"
        };

        let mut stmt = conn.prepare(query)?;
        let analog_data = stmt.query_map([], |row| {
            Ok(AnalogInputsOutputs {
                id: row.get(0)?,
                analog_input_0: row.get(1)?,
                analog_input_1: row.get(2)?,
                analog_input_2: row.get(3)?,
                analog_input_3: row.get(4)?,
                analog_input_4: row.get(5)?,
                analog_input_5: row.get(6)?,
                analog_input_6: row.get(7)?,
                analog_input_7: row.get(8)?,
                analog_output_0: row.get(9)?,
                analog_output_1: row.get(10)?,
                analog_output_2: row.get(11)?,
                analog_output_3: row.get(12)?,
                analog_output_4: row.get(13)?,
                analog_output_5: row.get(14)?,
                analog_output_6: row.get(15)?,
                analog_output_7: row.get(16)?,
                timestamp: row.get(17)?,
            })
        })?;
        analog_data.collect()
    }

    async fn get_digital_inputs_outputs(&self, latest: bool) -> Result<Vec<DigitalInputsOutputs>> {
        let conn = self.iov_conn.lock().await;
        let query = if latest {
            "SELECT * FROM digital_inputs_outputs ORDER BY timestamp DESC LIMIT 1"
        } else {
            "SELECT * FROM digital_inputs_outputs"
        };

        let mut stmt = conn.prepare(query)?;
        let digital_data = stmt.query_map([], |row| {
            Ok(DigitalInputsOutputs {
                id: row.get(0)?,
                digital_input_0: row.get(1)?,
                digital_input_1: row.get(2)?,
                digital_input_2: row.get(3)?,
                digital_input_3: row.get(4)?,
                digital_input_4: row.get(5)?,
                digital_input_5: row.get(6)?,
                digital_input_6: row.get(7)?,
                digital_input_7: row.get(8)?,
                digital_output_0: row.get(9)?,
                digital_output_1: row.get(10)?,
                digital_output_2: row.get(11)?,
                digital_output_3: row.get(12)?,
                digital_output_4: row.get(13)?,
                digital_output_5: row.get(14)?,
                digital_output_6: row.get(15)?,
                digital_output_7: row.get(16)?,
                timestamp: row.get(17)?,
            })
        })?;
        digital_data.collect()
    }

    async fn get_variables(&self, latest: bool) -> Result<Vec<Variables>> {
        let conn = self.iov_conn.lock().await;
        let query = if latest {
            "SELECT * FROM variables ORDER BY timestamp DESC LIMIT 1"
        } else {
            "SELECT * FROM variables"
        };

        let mut stmt = conn.prepare(query)?;
        let variables_data = stmt.query_map([], |row| {
            Ok(Variables {
                id: row.get(0)?,
                kp: row.get(1)?,
                ki: row.get(2)?,
                kd: row.get(3)?,
                system_state: row.get(4)?,
                timestamp: row.get(5)?,
            })
        })?;
        variables_data.collect()
    }

}

// Add this function after the DbConn implementation
fn validate_api_key(apikey: &str) -> bool {
    let conn = Connection::open("db/users.db").unwrap();
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM users WHERE apikey = ?1").unwrap();
    let count: i64 = stmt.query_row([apikey], |row| row.get(0)).unwrap_or(0);
    count > 0
}

#[derive(Serialize)]
struct AnalogInputsOutputs {
    id: i32,
    analog_input_0: f64,
    analog_input_1: f64,
    analog_input_2: f64,
    analog_input_3: f64,
    analog_input_4: f64,
    analog_input_5: f64,
    analog_input_6: f64,
    analog_input_7: f64,
    analog_output_0: f64,
    analog_output_1: f64,
    analog_output_2: f64,
    analog_output_3: f64,
    analog_output_4: f64,
    analog_output_5: f64,
    analog_output_6: f64,
    analog_output_7: f64,
    timestamp: String,
}

#[derive(Serialize)]
struct DigitalInputsOutputs {
    id: i32,
    digital_input_0: bool,
    digital_input_1: bool,
    digital_input_2: bool,
    digital_input_3: bool,
    digital_input_4: bool,
    digital_input_5: bool,
    digital_input_6: bool,
    digital_input_7: bool,
    digital_output_0: bool,
    digital_output_1: bool,
    digital_output_2: bool,
    digital_output_3: bool,
    digital_output_4: bool,
    digital_output_5: bool,
    digital_output_6: bool,
    digital_output_7: bool,
    timestamp: String,
}

#[derive(Serialize)]
struct Variables {
    id: i32,
    kp: f64,
    ki: f64,
    kd: f64,
    system_state: String,
    timestamp: String,
}

#[allow(dead_code)]
struct ApiKey(String);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ApiKey {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        if let Some(api_key) = request.headers().get_one("x-api-key") {
            if validate_api_key(api_key) {
                return Outcome::Success(ApiKey(api_key.to_string()));
            }
        }
        Outcome::Error((rocket::http::Status::Unauthorized, ()))
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

#[get("/api/analog_inputs_outputs?<latest>")]
async fn get_analog_data(
    _key: ApiKey,
    latest: Option<bool>,
    state: &State<AppState>
) -> Json<Vec<AnalogInputsOutputs>> {
    let data = state.db_conn.get_analog_inputs_outputs(latest.unwrap_or(false))
        .await
        .unwrap_or_default();
    Json(data)
}

#[get("/api/digital_inputs_outputs?<latest>")]
async fn get_digital_data(
    _key: ApiKey,
    latest: Option<bool>,
    state: &State<AppState>
) -> Json<Vec<DigitalInputsOutputs>> {
    let data = state.db_conn.get_digital_inputs_outputs(latest.unwrap_or(false))
        .await
        .unwrap_or_default();
    Json(data)
}

#[get("/api/variables?<latest>")]
async fn get_system_variables(
    _key: ApiKey,
    latest: Option<bool>,
    state: &State<AppState>
) -> Json<Vec<Variables>> {
    let data = state.db_conn.get_variables(latest.unwrap_or(false))
        .await
        .unwrap_or_default();
    Json(data)
}

// Launch the Rocket app
#[launch]
fn rocket() -> _ {
    let db_conn = DbConn::new(
        "db/users.db",
        "../Logic/IOV.db"  // Updated path to IOV.db
    ).expect("Failed to initialize database connections");
    
    let app_state = AppState {
        db_conn,
        active_sessions: Arc::new(Mutex::new(HashMap::new())),
        session_counter: Arc::new(AtomicU64::new(1)),
    };

    rocket::build()
        .manage(app_state)
        .mount("/", routes![
            index, login, process_login, dash, api, favicon, stats, stats_page,
            get_analog_data, get_digital_data, get_system_variables
        ])
        .mount("/", FileServer::from(relative!("templates")))
}