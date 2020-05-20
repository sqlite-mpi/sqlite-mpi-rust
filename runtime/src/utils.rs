use std::time::{Duration};
use uuid::Uuid;

use regex::Regex;

pub fn ms(ms: u64) -> Duration {
    Duration::from_millis(ms)
}

#[derive(Debug)]
pub enum UuidInvalid {
    Format,
    Error(uuid::parser::ParseError)
}


//static ref RE: Regex = Regex::new("...").unwrap();

//let re = Regex::new(r"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$").unwrap();

// Force clients to use the exact same format for uuids (do not normalize to a single format internally).
// - Allow hashmaps to be used using the exact same uuid keys on both client and server.
pub fn is_valid_uuid_v4_hypenated(id: &String) -> Result<(), UuidInvalid> {
    lazy_static! {
        // E.g: `936da01f-9abd-4d9d-80c7-02af85c822a8`
        static ref RE: Regex = Regex::new(r"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$").unwrap();
    }

    match Uuid::parse_str(id) {
        Ok(_) => {
            if RE.is_match(id) {
                return Ok(())
            }
            Err(UuidInvalid::Format)
        },
        Err(e) => Err(UuidInvalid::Error(e))
    }
}
