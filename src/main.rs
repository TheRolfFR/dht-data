use std::io::Write;
use std::path::PathBuf;
#[cfg(feature = "cron_get")]
use std::time::Duration;
#[cfg(feature = "cron_get")]
use std::thread::sleep;

#[cfg(feature = "pm2logs")]
use std::fs::File;

use chrono::Utc;
#[cfg(feature = "cron_get")]
use reqwest::blocking;
use rouille::input::json_input;
use rouille::{router, Server as RouilleServer, try_or_400, ResponseBody, Response};
use dirs::home_dir;

use std::thread;
use std::sync::mpsc::sync_channel as channel;
use std::sync::{Arc, Mutex};
use std::convert::From;

pub mod data;
use data::*;

mod file_records;
use file_records::load_records;
use std::env;

mod utils;
use utils::first_and_last_character;

use crate::file_records::save_records;

const DEFAULT_PATH: &str = ".local/dht-data.json";
const DEFAULT_PORT: &str = "8888";
const DEFAULT_PASSWORD: &str = "password";

#[cfg(feature = "pm2logs")]
const PM2_LOG_PORT: &str = "/root/.pm2/logs/DHT-DATA-error.log";

fn main() -> Result<(), Box<dyn std::error::Error>>  {
    const VERSION: &str = env!("CARGO_PKG_VERSION");

    let args: Vec<String> = env::args().collect();

    #[cfg(feature = "cron_get")]
    let sensor_url = args.get(1).map(|u| u.to_string()).expect("No Sensor URL provided");

    //* RECORDS
    let path = args.get(2)
    .map(|p| PathBuf::from(p))
    .unwrap_or_else(|| {
        let mut path = home_dir().unwrap();
        path.push(PathBuf::from(DEFAULT_PATH));
        path
    });
    let records = load_records(&path);
    println!("Saving records to '{}'", path.to_str().unwrap());
    
    //* SERVER ADDRESS
    let server_port = args.get(3).map(|s| s.as_str()).unwrap_or(DEFAULT_PORT);
    let socket_addr = "0.0.0.0:".to_string() + server_port;
    print!("Starting server v{VERSION} at: '{socket_addr}' ");
    std::io::stdout().flush().ok();

    //* SERVER PASSWORD
    let server_password = args.get(4).map(|s| s.as_str()).unwrap_or(DEFAULT_PASSWORD).to_string();
    if server_password == DEFAULT_PASSWORD {
        print!("Starting server with default password {DEFAULT_PASSWORD}");
    } else {
        let hidden = first_and_last_character(server_password.as_str());
        print!("Starting server with default password {hidden} ");
    }
    std::io::stdout().flush().ok();

    let acc: Arc<Mutex<Vec<Record<SensorResponse>>>> = Arc::new(Mutex::new(records));
    let acc_write = acc.clone();

    let (tx, rx) = channel(2);
    let tx_clone = tx.clone();

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

    #[cfg(feature = "cron_get")]
    {
        let ten_min = Duration::from_secs(600); // 600 = 10min
        thread::spawn(move ||  {
            loop {
                let resp = blocking::get(sensor_url.clone());
                match resp {
                    Ok(res) => {
                        let val: SensorResponse = res.json().unwrap();
            
                        tx.send(val).unwrap();
                        sleep(ten_min);
                    },
                    Err(err) => {
                        let safe_err = err.without_url();
                        eprintln!("Failed to get sensor: {safe_err}");
                        sleep(ten_min);
                    }
                }
            }
        });
    }

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
            (GET) (/logs) => {
                #[cfg(feature = "pm2logs")]
                {
                    File::open(PM2_LOG_PORT)
                    .map(|file| Response::from_file("text/plain", file))
                    .unwrap_or_else(|_| rouille::Response::empty_404())
                }
                #[cfg(not(feature = "pm2logs"))]
                rouille::Response::empty_404()
            },
            (GET) (/last) => {
                let mut vec = acc.lock().unwrap().clone().iter().map(|e| RecordEntry::from(e)).collect::<Vec<RecordEntry>>();
                vec.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
                rouille::Response::json(&vec.last().unwrap())
            },
            (POST) (/new) => {
                let data: SensorPayload = try_or_400!(json_input(request));

                if data.password != server_password {
                    Response {
                        status_code: 403,
                        headers: vec![],
                        data: ResponseBody::empty(),
                        upgrade: None,
                    }
                } else {
                    let temperature = data.temperature;
                    let humidity = data.humidity;
                    let val = SensorResponse::new(temperature, humidity);

                    tx_clone.send(val).unwrap();

                    Response {
                        status_code: 201,
                        headers: vec![],
                        data: ResponseBody::empty(),
                        upgrade: None,
                    }
                }
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
