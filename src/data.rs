
use std::fmt::Debug;
use chrono::{DateTime, Utc, serde::{ts_seconds}};
use chrono_tz;
use serde::{Serialize, Deserialize};
use std::convert::From;

#[derive(Serialize, serde::Deserialize, Debug, Clone)]
pub struct DHT11  {
    pub temp: f32,
    pub humi: f32
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SensorResponse {
    pub temp: f32,
    pub temperature: f32,
    pub humidity: f32,
    pub dht11: DHT11
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RecordEntry {
    pub temperature: f32,
    pub humidity: f32,
    pub timestamp: i64,
    pub date: String
}

impl From<&Record<SensorResponse>> for RecordEntry {
    fn from(item: &Record<SensorResponse>) -> Self {
        Self {
            temperature: item.value.temperature,
            humidity: item.value.humidity,
            timestamp: item.date.timestamp(),
            date: item.date.with_timezone(&chrono_tz::Tz::Europe__Paris).to_rfc2822()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Record<T> {
    pub value: T,
    #[serde(with = "ts_seconds")]
    pub date: DateTime<Utc>
}
