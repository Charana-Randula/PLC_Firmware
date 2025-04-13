use rusqlite::{params, Connection, Result};
use chrono::Local;

// Function to create tables in the database
pub fn create_tables(conn: &Connection) -> Result<()> {
    // Create table for analog inputs and outputs
    conn.execute(
        "CREATE TABLE IF NOT EXISTS analog_inputs_outputs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            analog_input_0 REAL,
            analog_input_1 REAL,
            analog_input_2 REAL,
            analog_input_3 REAL,
            analog_input_4 REAL,
            analog_input_5 REAL,
            analog_input_6 REAL,
            analog_input_7 REAL,
            analog_output_0 REAL,
            analog_output_1 REAL,
            analog_output_2 REAL,
            analog_output_3 REAL,
            analog_output_4 REAL,
            analog_output_5 REAL,
            analog_output_6 REAL,
            analog_output_7 REAL,
            timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    // Create table for digital inputs and outputs
    conn.execute(
        "CREATE TABLE IF NOT EXISTS digital_inputs_outputs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            digital_input_0 INTEGER,
            digital_input_1 INTEGER,
            digital_input_2 INTEGER,
            digital_input_3 INTEGER,
            digital_input_4 INTEGER,
            digital_input_5 INTEGER,
            digital_input_6 INTEGER,
            digital_input_7 INTEGER,
            digital_output_0 INTEGER,
            digital_output_1 INTEGER,
            digital_output_2 INTEGER,
            digital_output_3 INTEGER,
            digital_output_4 INTEGER,
            digital_output_5 INTEGER,
            digital_output_6 INTEGER,
            digital_output_7 INTEGER,
            timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    // Create table for system variables
    conn.execute(
        "CREATE TABLE IF NOT EXISTS variables (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            kp REAL,
            ki REAL,
            kd REAL,
            system_state TEXT,
            timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    Ok(())
}

// Function to limit the number of rows in a table to 30
fn enforce_row_limit(conn: &Connection, table_name: &str) -> Result<()> {
    // Delete rows if the table has more than 30 entries
    conn.execute(
        &format!(
            "DELETE FROM {} WHERE id IN (
                SELECT id FROM {} ORDER BY timestamp ASC LIMIT (
                    SELECT MAX(0, COUNT(*) - 30) FROM {}
                )
            )",
            table_name, table_name, table_name
        ),
        [],
    )?;
    Ok(())
}

// Function to insert simulation data into the database
pub fn insert_data(conn: &Connection, system: &crate::hvac_system::HVACSystem) -> Result<()> {
    // Get the current local timestamp
    let local_timestamp = Local::now().to_rfc3339();

    // Insert analog input/output data into the database
    conn.execute(
        "INSERT INTO analog_inputs_outputs (
            analog_input_0, analog_input_1, analog_input_2, analog_input_3,
            analog_input_4, analog_input_5, analog_input_6, analog_input_7,
            analog_output_0, analog_output_1, analog_output_2, analog_output_3,
            analog_output_4, analog_output_5, analog_output_6, analog_output_7,
            timestamp
        ) VALUES (
            ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?
        )",
        params![
            system.analog_input_0, system.analog_input_1, system.analog_input_2, system.analog_input_3,
            system.analog_input_4, system.analog_input_5, system.analog_input_6, system.analog_input_7,
            system.analog_output_0, system.analog_output_1, system.analog_output_2, system.analog_output_3,
            system.analog_output_4, system.analog_output_5, system.analog_output_6, system.analog_output_7,
            local_timestamp
        ],
    )?;

    // Enforce row limit for analog_inputs_outputs table
    enforce_row_limit(conn, "analog_inputs_outputs")?;

    // Insert digital input/output data into the database
    conn.execute(
        "INSERT INTO digital_inputs_outputs (
            digital_input_0, digital_input_1, digital_input_2, digital_input_3,
            digital_input_4, digital_input_5, digital_input_6, digital_input_7,
            digital_output_0, digital_output_1, digital_output_2, digital_output_3,
            digital_output_4, digital_output_5, digital_output_6, digital_output_7,
            timestamp
        ) VALUES (
            ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?
        )",
        params![
            system.digital_input_0 as i32, system.digital_input_1 as i32, system.digital_input_2 as i32, system.digital_input_3 as i32,
            system.digital_input_4 as i32, system.digital_input_5 as i32, system.digital_input_6 as i32, system.digital_input_7 as i32,
            system.digital_output_0 as i32, system.digital_output_1 as i32, system.digital_output_2 as i32, system.digital_output_3 as i32,
            system.digital_output_4 as i32, system.digital_output_5 as i32, system.digital_output_6 as i32, system.digital_output_7 as i32,
            local_timestamp
        ],
    )?;

    // Enforce row limit for digital_inputs_outputs table
    enforce_row_limit(conn, "digital_inputs_outputs")?;

    // Check if the variables table is empty
    let row_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM variables",
        [],
        |row| row.get(0),
    )?;

    // Only insert into the variables table if it is empty
    if row_count == 0 {
        conn.execute(
            "INSERT INTO variables (
                kp, ki, kd, system_state, timestamp
            ) VALUES (
                ?, ?, ?, ?, ?
            )",
            params![
                system.kp, system.ki, system.kd, system.system_state, local_timestamp
            ],
        )?;
    }

    Ok(())
}