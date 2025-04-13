use rusqlite::Connection;
use crate::hvac_system::HVACSystem;
use crate::connect_db::insert_data;
use std::thread::sleep;
use std::time::Duration;

pub fn run_simulation(conn: &Connection, system: &mut HVACSystem) -> rusqlite::Result<()> {
    loop {
        // Apply PID control
        system.apply_pid();

        // Simulate input changes (temperature, humidity, airflow)
        system.simulate();

        // Insert data into the database
        insert_data(conn, system)?;

        // Print the status of the variables
        println!("{:?}", system);

        // Wait for 5 seconds before the next iteration
        sleep(Duration::new(3, 0));
    }
}
