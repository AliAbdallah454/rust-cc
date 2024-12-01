use rocket::serde::Serialize;


#[derive(Debug, Clone, Serialize)]
pub struct Transaction {
    pub source: String,
    pub destination: String,
    pub begining: u64,
    pub ending: u64,
}

impl Transaction {
    pub fn new(source: String, destination: String, begining: u64, ending: u64) -> Self {
        return Transaction {
            source,
            destination,
            begining,
            ending,
        };
    }
}