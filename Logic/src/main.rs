mod hvac_system;
mod connect_db;
mod simulation;

use hvac_system::HVACSystem;
use connect_db::create_tables;
use simulation::run_simulation;

fn main() -> rusqlite::Result<()> {
    // Connect to the SQLite database
    let conn = rusqlite::Connection::open("IOV.db")?;

    // Create tables if they do not exist
    create_tables(&conn)?;

    // Create a new HVAC system instance
    let mut system = HVACSystem::new();

    // Run the simulation
    run_simulation(&conn, &mut system)
}
