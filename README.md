# An Open-Source PLC Firmware based on Rust and Rocket.rs

## 🛠️ Installation Guide

### ✅ System Requirements

* Raspberry Pi (recommended) or any Linux system with GPIO access
* Rust toolchain (`cargo`, `rustc`, etc.)
* Git
* Internet access (for pulling dependencies)

---

### 📦 1. Install Prerequisites

#### 🔧 Install Rust

If Rust is not already installed:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

After installation:

```bash
source $HOME/.cargo/env
rustup update
```

#### 📁 Install Other Tools

```bash
sudo apt update
sudo apt install git build-essential pkg-config libssl-dev libsqlite3-dev
```

For GPIO support on Raspberry Pi:

```bash
sudo apt install libgpiod-dev
```

---

### 📂 2. Clone the Repository

```bash
git clone https://github.com/Charana-Randula/PLC_Firmware/tree/main
cd EEP1
```

---

### ⚙️ 3. Configure Environment (Optional)

If you use environment variables (e.g., for database URLs, API keys, etc.), create a `.env` file in the project root:

```env
ROCKET_ADDRESS=0.0.0.0
ROCKET_PORT=8000
API_KEY_SECRET=your-secret-key
```

Or modify directly in `Rocket.toml`.

---

### 🏗️ 4. Build the Project

In the root directory:

```bash
cargo build --release
```

This compiles the project in release mode for better performance.

---

### ▶️ 5. Run the Server

To start the web server:

```bash
cargo run --release
```

If using systemd or supervisor for deployment, you can daemonize it — ask if you want a unit file.

---

### 🌐 6. Access the Web Interface or API

Once the server is running, open a browser or API tool and go to:

```
http://localhost:8000
```

Example API request:

```bash
curl "http://localhost:8000/api/analog_inputs_outputs?latest=true"
```

---

### 🧪 7. Test GPIO (Optional)

If you're using GPIO functionality, you may need to run the binary with `sudo`:

```bash
sudo cargo run --release
```

Test that your pins are accessible (especially on a Raspberry Pi using `/dev/gpiochip0`).

---

## ⚠️ Common Build Errors & Fixes

### ❌ Error: `cannot find -lsqlite3` or `libsqlite3-sys` build failure

**Cause:** Missing SQLite development headers.([Stack Overflow][1])

**Fix:**

```bash
sudo apt install libsqlite3-dev
```

This installs the necessary C headers and libraries required by `libsqlite3-sys`.

### ❌ Error: `pkg-config` not found

**Cause:** `pkg-config` is missing, which is used to locate libraries during the build process.([Docs.rs][2])

**Fix:**

```bash
sudo apt install pkg-config
```

### ❌ Error: `libssl-dev` missing (for TLS support)

**Cause:** Missing OpenSSL development libraries.

**Fix:**

```bash
sudo apt install libssl-dev
```

---

## 📚 Additional Resources

* [libsqlite3-sys Documentation](https://docs.rs/libsqlite3-sys/latest/libsqlite3_sys/)
* [rusqlite GitHub Repository](https://github.com/rusqlite/rusqlite)

---

Let me know if you need further assistance or additional information!

[1]: https://stackoverflow.com/questions/65559230/how-to-fix-cannot-find-lsqlite3-error-when-deploying-rust-app-to-heroku?utm_source=chatgpt.com "How to fix \"cannot find -lsqlite3\" error when deploying Rust app to ..."
[2]: https://docs.rs/crate/libsqlite3-sys/latest?utm_source=chatgpt.com "libsqlite3-sys 0.33.0 - Docs.rs"
