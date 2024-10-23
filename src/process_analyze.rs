extern crate sysinfo;
use chrono::{FixedOffset, Utc};
use serde_json::json;
use std::collections::{HashMap, HashSet};
use std::io;
use sysinfo::System;
use tokio::time::{sleep, Duration};

fn set_record_period() -> i32 {
    let mut input = String::new();
    println!("What's the record period in secs do you want to record the process usage?");

    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input");

    input.trim().parse().unwrap_or_else(|_| {
        println!("Invalid input. Please enter a valid integer.");
        set_record_period()
    })
}

fn set_return_period() -> i32 {
    let mut input = String::new();
    println!("What's the return period in secs do you want to get the total process usage?");
    println!("The number of records in the returned data is the result of dividing the upload period by the record period");
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input");

    input.trim().parse().unwrap_or_else(|_| {
        println!("Invalid input. Please enter a valid integer.");
        set_record_period()
    })
}

pub fn return_time() -> String {
    Utc::now()
        .with_timezone(&FixedOffset::east_opt(8 * 3600).unwrap())
        .format("%Y-%m-%d %H:%M:%S")
        .to_string()
}

#[derive(Debug, Clone)]
struct ProcessInfo {
    name: String,
    open_time: String,
    close_time: String,
}

impl ProcessInfo {
    fn new(name: String) -> Self {
        ProcessInfo {
            name,
            open_time: return_time(),
            close_time: "Not yet!".to_string(),
        }
    }

    fn set_close_time(&mut self) {
        self.close_time = return_time();
    }
}

fn add_process_open_time(
    process_name: String,
    process_id: String,
    process_analyzes: &mut HashMap<String, ProcessInfo>,
) {
    process_analyzes
        .entry(process_id.clone())
        .or_insert(ProcessInfo::new(process_name));
}

fn add_process_close_time(
    process_id: &String,
    new_record_set: &HashSet<String>,
    process_analyzes: &mut HashMap<String, ProcessInfo>,
) {
    if !new_record_set.contains(process_id) {
        if let Some(record) = process_analyzes.get_mut(process_id) {
            record.set_close_time();
        }
    }
}

fn process_analyze(
    sys: &mut System,
    process_analyzes: &mut HashMap<String, ProcessInfo>,
    pre_record_set: &mut HashSet<String>,
) {
    sys.refresh_all();

    let mut new_record_set = HashSet::new();

    for (process_id, process) in sys.processes() {
        let process_name = process.name().to_string_lossy().to_string();
        let process_id = process_id.to_string();

        new_record_set.insert(process_id.clone());

        add_process_open_time(process_name, process_id, process_analyzes);
    }

    for process_id in pre_record_set.iter() {
        add_process_close_time(process_id, &new_record_set, process_analyzes);
    }

    *pre_record_set = new_record_set;
}

fn summarize_processes(
    process_analyzes: &HashMap<String, ProcessInfo>,
) -> HashMap<String, serde_json::Value> {
    let mut process_total_usage: HashMap<String, serde_json::Value> = HashMap::new();

    for (_, process_info) in process_analyzes {
        let entry = process_total_usage
            .entry(process_info.name.clone())
            .or_insert_with(|| {
                json!({
                    "times": 0,
                    "details": Vec::<serde_json::Value>::new(),
                })
            });

        if let Some(times) = entry["times"].as_i64() {
            entry["times"] = json!(times + 1);
        }

        let details_array = entry["details"].as_array_mut().unwrap();
        details_array.push(json!({
            "open": process_info.open_time,
            "close": process_info.close_time,
        }));
    }

    process_total_usage
}

pub async fn analyze_process_status() {
    let period_of_record = set_record_period();
    let period_of_upload = set_return_period();
    let total_times = period_of_upload / period_of_record;
    println!("Starting to record, please wait...");
    let mut process_analyzes: HashMap<String, ProcessInfo> = HashMap::new();
    let mut pre_record_set: HashSet<String> = HashSet::new();

    let mut sys = System::new_all();
    loop {
        for _ in 0..total_times {
            process_analyze(&mut sys, &mut process_analyzes, &mut pre_record_set);
            sleep(Duration::from_secs(period_of_record as u64)).await;
        }

        let process_total_usage = summarize_processes(&process_analyzes);
        println!("{:#?}", process_total_usage);
    }
}
