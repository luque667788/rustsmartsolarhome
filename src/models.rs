use serde::{ Deserialize, Serialize };
use std::fmt;

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub enum ActionMqtt {
    State(bool),
    Get,
    setmanualmode(bool),
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct User {
    pub name: String,
    pub pass: String,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct DayPw {
    pub value: String,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct LogData {
    pub timeon: String,
    pub currenttimehours: String,
}

impl fmt::Display for LogData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "timeon: {}\n \n current time: {}", self.timeon, self.currenttimehours)
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ParamsJson {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mininterval: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mintimeon: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minpw: Option<i32>,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct RelayMqtt {
    pub value: String,
    pub mode: bool,
}
#[derive(Clone, Default, Serialize, Deserialize,PartialEq)]
pub struct RebootMqtt {
    pub value: i64,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct CurrentPw {
    pub value: String,
}
