use std::io::Write;
use std::path::PathBuf;
use std::time::Duration;
use std::thread::sleep;
use chrono::Utc;
use reqwest::blocking;
use rouille::{router, Server as RouilleServer};
use dirs::home_dir;

use std::thread;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::convert::From;

pub mod data;
use data::*;

mod file_records;
use file_records::load_records;
use std::env;

use crate::file_records::save_records;

const DEFAULT_PATH: &str = ".local/dht-data.json";
const DEFAULT_PORT: &str = "8888";

fn main() -> Result<(), Box<dyn std::error::Error>>  {
    const VERSION: &str = env!("CARGO_PKG_VERSION");

    let mut args = env::args();
    let sensor_url = args.nth(1).expect("No Sensor URL provided");

    //* RECORDS
    let path = args.nth(2)
    .map(|p| PathBuf::from(p))
    .unwrap_or_else(|| {
        let mut path = home_dir().unwrap();
        path.push(PathBuf::from(DEFAULT_PATH));
        path
    });
    let records = load_records(&path);
    
    //* SERVER ADDRESS
    let server_port = args.nth(3).unwrap_or(DEFAULT_PORT.to_string());
    let socket_addr = "0.0.0.0:".to_string() + &server_port;
    print!("Starting server v{VERSION} at: {socket_addr} ");
    std::io::stdout().flush().ok();

    let acc: Arc<Mutex<Vec<Record<SensorResponse>>>> = Arc::new(Mutex::new(records));
    let acc_write = acc.clone();
    let ten_min = Duration::from_secs(600); // 600 = 10min
    let one_min = Duration::from_secs(60);

    let (tx, rx) = channel();

    thread::spawn(move || {
        let max: usize = 144;
        let mut i: usize = 0;
        loop {
            let val = rx.recv().unwrap();
            
            let mut acc = acc_write.lock().unwrap();
            if acc.len() < max {
                acc.push(Record {
                    value: val,
                    date: Utc::now()
                });
            } else {
                let el = &mut acc[i];
                *el = Record {
                    value: val,
                    date: Utc::now()
                };
            }
            save_records(&path, &acc);
            i = (i+1) % max;
        }
    });

    thread::spawn(move ||  {
        loop {
            let resp = blocking::get(&sensor_url);
            match resp {
                Ok(res) => {
                    let val: SensorResponse = res.json().unwrap();
        
                    tx.send(val).unwrap();
                    sleep(ten_min);
                },
                Err(err) => {
                    let safe_err = err.without_url();
                    eprintln!("Failed to get sensor: {safe_err}");
                    sleep(one_min);
                }
            }
        }
    });

    RouilleServer::new(socket_addr, move |request| {
        router!(request,
            (GET) (/) => {
                let descending = request.raw_url().contains("descending");

                let mut vec = acc.lock().unwrap().clone().iter().map(|e| RecordEntry::from(e)).collect::<Vec<RecordEntry>>();

                if descending {
                    vec.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
                } else {
                    vec.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
                }
                
                rouille::Response::json(&vec)
            },
            _ => rouille::Response::empty_404()
        )
    })
    .map(|v| {
        println!("✅!");
        v
    })
    .expect("Failed to start server")
    .run();

    panic!("The server socket closed unexpectedly");
}
