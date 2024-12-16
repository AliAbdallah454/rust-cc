use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize,Debug)]
pub struct Exclusive
{
    pub keys_hash:HashMap<String, u128>,
    pub wb:HashMap<String,String>
}

impl Exclusive
{
    pub fn new(keys_hash:HashMap<String, u128>,wb:HashMap<String,String>)->Self
    {
        Self { keys_hash, wb }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RedirectInfo {
    pub exclusive: Exclusive,
    pub destination: String
}