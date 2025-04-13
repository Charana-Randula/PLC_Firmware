use rand::Rng;
use rusqlite::{Connection, Result};

#[derive(Debug)]
pub struct HVACSystem {
    pub analog_input_0: f32, // Zone 1 Temperature
    pub analog_input_1: f32, // Zone 2 Temperature
    pub analog_input_2: f32, // Zone 3 Temperature
    pub analog_input_3: f32, // Zone 4 Temperature
    pub analog_input_4: f32, // Zone 1 Humidity
    pub analog_input_5: f32, // Zone 2 Humidity
    pub analog_input_6: f32, // Airflow
    pub analog_input_7: f32, // Reserved

    pub analog_output_0: f32, // Adjusted Zone 1 Temperature
    pub analog_output_1: f32, // Adjusted Zone 2 Temperature
    pub analog_output_2: f32, // Adjusted Zone 3 Temperature
    pub analog_output_3: f32, // Adjusted Zone 4 Temperature
    pub analog_output_4: f32, // Adjusted Zone 1 Humidity
    pub analog_output_5: f32, // Adjusted Zone 2 Humidity
    pub analog_output_6: f32, // Adjusted Airflow
    pub analog_output_7: f32, // Reserved

    // Digital inputs (representing sensors)
    pub digital_input_0: bool, // Zone 1 Valve Status
    pub digital_input_1: bool, // Zone 2 Valve Status
    pub digital_input_2: bool, // Zone 3 Valve Status
    pub digital_input_3: bool, // Zone 4 Valve Status
    pub digital_input_4: bool, // Emergency Shutdown
    pub digital_input_5: bool, // Power Supply Status
    pub digital_input_6: bool, // Maintenance Mode
    pub digital_input_7: bool, // Reserved

    // Digital outputs (representing actuators)
    pub digital_output_0: bool, // Zone 1 Heating Valve
    pub digital_output_1: bool, // Zone 2 Heating Valve
    pub digital_output_2: bool, // Zone 3 Heating Valve
    pub digital_output_3: bool, // Zone 4 Heating Valve
    pub digital_output_4: bool, // Zone 1 Cooling Valve
    pub digital_output_5: bool, // Zone 2 Cooling Valve
    pub digital_output_6: bool, // Zone 3 Cooling Valve
    pub digital_output_7: bool, // Zone 4 Cooling Valve

    pub kp: f32, // PID Proportional Gain
    pub ki: f32, // PID Integral Gain
    pub kd: f32, // PID Derivative Gain
    pub system_state: String,
}

impl HVACSystem {
    // Constructor for HVACSystem
    pub fn new() -> HVACSystem {
        // Attempt to connect to the database
        let conn = Connection::open("IOV.db");

        // Default values
        let mut kp = 1.0;
        let mut ki = 0.1;
        let mut kd = 0.05;
        let mut system_state = String::from("Operating");

        if let Ok(conn) = conn {
            // Query the database for existing values
            let query_result: Result<(f32, f32, f32, String)> = conn.query_row(
                "SELECT kp, ki, kd, system_state FROM variables ORDER BY timestamp DESC LIMIT 1",
                [],
                |row| {
                    Ok((
                        row.get(0)?, // kp
                        row.get(1)?, // ki
                        row.get(2)?, // kd
                        row.get(3)?, // system_state
                    ))
                },
            );

            if let Ok((db_kp, db_ki, db_kd, db_system_state)) = query_result {
                kp = db_kp;
                ki = db_ki;
                kd = db_kd;
                system_state = db_system_state;
            }
        }

        HVACSystem {
            analog_input_0: 22.0,
            analog_input_1: 21.8,
            analog_input_2: 22.1,
            analog_input_3: 22.0,
            analog_input_4: 50.0,
            analog_input_5: 48.0,
            analog_input_6: 300.0,
            analog_input_7: 0.0, // Reserved
            analog_output_0: 22.0,
            analog_output_1: 22.0,
            analog_output_2: 22.0,
            analog_output_3: 22.0,
            analog_output_4: 50.0,
            analog_output_5: 48.0,
            analog_output_6: 300.0,
            analog_output_7: 0.0, // Reserved

            // Initialize digital inputs
            digital_input_0: false,
            digital_input_1: false,
            digital_input_2: false,
            digital_input_3: false,
            digital_input_4: false,
            digital_input_5: true,  // Power supply on by default
            digital_input_6: false,
            digital_input_7: false,

            // Initialize digital outputs
            digital_output_0: false,
            digital_output_1: false,
            digital_output_2: false,
            digital_output_3: false,
            digital_output_4: false,
            digital_output_5: false,
            digital_output_6: false,
            digital_output_7: false,

            kp,
            ki,
            kd,
            system_state,
        }
    }

    pub fn apply_pid(&mut self) {
        // Simple PID control logic for temperature and humidity
        for i in 0..4 {
            let temp_input = match i {
                0 => self.analog_input_0,
                1 => self.analog_input_1,
                2 => self.analog_input_2,
                3 => self.analog_input_3,
                _ => 0.0,
            };

            let temp_output = match i {
                0 => &mut self.analog_output_0,
                1 => &mut self.analog_output_1,
                2 => &mut self.analog_output_2,
                3 => &mut self.analog_output_3,
                _ => unreachable!(),
            };

            // Apply PID formula
            *temp_output = temp_input
                + self.kp * (temp_input - *temp_output)
                + self.ki * temp_input
                + self.kd * (temp_input - *temp_output);

            // Clamp the output to a reasonable range (e.g., 10.0 to 35.0 for temperature)
            *temp_output = temp_output.clamp(10.0, 35.0);
        }
    }

    pub fn process_digital_inputs(&mut self) {
        // Emergency shutdown: turn off all valves
        if self.digital_input_4 {
            self.digital_output_0 = false;
            self.digital_output_1 = false;
            self.digital_output_2 = false;
            self.digital_output_3 = false;
            self.digital_output_4 = false;
            self.digital_output_5 = false;
            self.digital_output_6 = false;
            self.digital_output_7 = false;
            return;
        }

        // Process heating and cooling valves based on zone status
        self.digital_output_0 = self.digital_input_0; // Zone 1 Heating
        self.digital_output_1 = self.digital_input_1; // Zone 2 Heating
        self.digital_output_2 = self.digital_input_2; // Zone 3 Heating
        self.digital_output_3 = self.digital_input_3; // Zone 4 Heating

        self.digital_output_4 = !self.digital_input_0; // Zone 1 Cooling
        self.digital_output_5 = !self.digital_input_1; // Zone 2 Cooling
        self.digital_output_6 = !self.digital_input_2; // Zone 3 Cooling
        self.digital_output_7 = !self.digital_input_3; // Zone 4 Cooling
    }

    pub fn simulate(&mut self) {
        let mut rng = rand::thread_rng();

        // Randomize analog inputs
        self.analog_input_0 = rng.gen_range(15.0..=30.0); // Zone 1 Temperature
        self.analog_input_1 = rng.gen_range(15.0..=30.0); // Zone 2 Temperature
        self.analog_input_2 = rng.gen_range(15.0..=30.0); // Zone 3 Temperature
        self.analog_input_3 = rng.gen_range(15.0..=30.0); // Zone 4 Temperature
        self.analog_input_4 = rng.gen_range(30.0..=70.0); // Zone 1 Humidity
        self.analog_input_5 = rng.gen_range(30.0..=70.0); // Zone 2 Humidity
        self.analog_input_6 = rng.gen_range(200.0..=500.0); // Airflow

        // Randomize digital inputs
        self.digital_input_0 = rng.gen_bool(0.5); // Zone 1 Valve
        self.digital_input_1 = rng.gen_bool(0.5); // Zone 2 Valve
        self.digital_input_2 = rng.gen_bool(0.5); // Zone 3 Valve
        self.digital_input_3 = rng.gen_bool(0.5); // Zone 4 Valve
        self.digital_input_4 = rng.gen_bool(0.1); // Emergency Shutdown (low probability)
        self.digital_input_5 = rng.gen_bool(0.9); // Power Supply Status (high probability)
        self.digital_input_6 = rng.gen_bool(0.2); // Maintenance Mode (medium probability)

        // Process inputs and apply PID control
        self.process_digital_inputs();
        self.apply_pid();
    }
}
