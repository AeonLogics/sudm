use std::env;
use std::path::PathBuf;

pub struct SudmConfig;

impl SudmConfig {
    pub const SCHEMA_PATH: &'static str = "surql_flux/schema";
    pub const FUNCTIONS_PATH: &'static str = "surql_flux/functions";
    pub const EVENTS_PATH: &'static str = "surql_flux/events";

    pub fn current_schema_path() -> PathBuf {
        let current_location = env::current_dir()
            .map_err(|e| format!("Could not read active terminal path: {}", e))
            .unwrap();
        current_location.join(Self::SCHEMA_PATH)
    }

    pub fn current_functions_path() -> PathBuf {
        let current_location = env::current_dir()
            .map_err(|e| format!("Could not read active terminal path: {}", e))
            .unwrap();
        current_location.join(Self::FUNCTIONS_PATH)
    }

    pub fn current_events_path() -> PathBuf {
        let current_location = env::current_dir()
            .map_err(|e| format!("Could not read active terminal path: {}", e))
            .unwrap();

        current_location.join(Self::EVENTS_PATH)
    }

    pub fn flux_schema_paths()  -> [PathBuf; 3] {
        [
            Self::current_schema_path(),
            Self::current_functions_path(),
            Self::current_events_path()
        ]
    }
}