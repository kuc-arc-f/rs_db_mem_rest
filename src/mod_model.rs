use serde::{Serialize, Deserialize};
use serde_json::{json, Value};
use std::env;
use std::io::{self, BufRead, Write};
//use dotenvy::dotenv;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct QueItem {
    pub sql: String,
    pub uuid: String,
}
