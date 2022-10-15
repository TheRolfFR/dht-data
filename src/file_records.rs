use std::{fs::{File}, io::Read, path::{PathBuf}};

use crate::data::{SensorResponse, Record};

pub fn load_records(path: &PathBuf) -> Vec<Record<SensorResponse>> {
    let cloned = path.clone(); // cannot use clone if AsRef<Path>
    let disp = cloned.display();

    let file: Option<File> = File::open(path)
    .map_err(|e| {
        eprintln!("Failed to open {disp}: {e}");
        e
    }).ok();

    file
    .and_then(|mut f| {
        let mut buff = String::new();
        f.read_to_string(&mut buff)
        .map_err(|e| {
            eprintln!("Failed to read file in buffer");
            e
        })
        .and(Ok(buff))
        .ok()
    })
    .and_then(|buff: String| -> Option<Vec<Record<SensorResponse>>> {
        serde_json::from_str(&buff)
        .map_err(|e| {
            eprintln!("Failed to transcript into records");
            e
        })
        .ok()
    })
    .unwrap_or(vec![])
}

pub fn save_records(path: &PathBuf, records: &Vec<Record<SensorResponse>>) {
    std::fs::write(
        path,
        serde_json::to_string_pretty(&records).unwrap(),
    ).unwrap();
}