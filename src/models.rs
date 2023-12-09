
use serde::{ Deserialize, Serialize };
use std::fmt;

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub enum ActionMqtt {
    State(bool),
    Get,
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
    pub totaltimeon: String,
    pub currenttimehours: String,
}

impl fmt::Display for LogData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "timeon: {}\n totaltimeon: {} \n current time: {}",
            self.timeon,
            self.totaltimeon,
            self.currenttimehours
        )
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct RelayMqtt {
    pub value: String,
}
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct RebootMqtt {
    pub value: i64,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct CurrentPw {
    pub value: String,
}