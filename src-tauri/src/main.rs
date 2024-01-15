// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// origin heartbeat modules
use std::error::Error;
use std::thread;

use bluest::{Adapter, AdvertisingDevice};
use futures_lite::stream::StreamExt;

fn handle_device(discovered_device: AdvertisingDevice) {
    if let Some(manufacturer_data) = discovered_device.adv_data.manufacturer_data {
        if manufacturer_data.company_id != 0x0157 {
            return;
        }
        let name = discovered_device
            .device
            .name()
            .unwrap_or(String::from("(unknown)"));
        let rssi = discovered_device.rssi.unwrap_or_default();
        let heart_rate = manufacturer_data.data[3];
        println!("{name} ({rssi}dBm) Heart Rate: {heart_rate:?}",);
        change_global_value(heart_rate);
    }
}

use once_cell::sync::Lazy;
use parking_lot::Mutex;
pub static GLOBAL_VALUE: Lazy<Mutex<String>> = Lazy::new(|| Mutex::new("".to_string()));
fn change_global_value(rate: u8) {
    // 因为使用的是Mutex的方式，但是这个方式有个弊端
    // Mutex<T>  MutexGuard<T>  运行时可能会死锁，可能会阻塞线程
    // 解释也就是Mutex是一种互斥锁，通常用于保证多个线程不会同时访问共享数据。
    // 而MutexGuard 是一个智能指针，超出作用域时会自动释放锁
    // 所以创建新的进程来进行赋值，这样在进程结束的时候 自动解锁；
    let handle = thread::spawn(move || {
        let mut _global_value = GLOBAL_VALUE.lock();
        *_global_value = rate.to_string()
    });
    handle.join().unwrap()
}
fn use_global_value() -> String {
    let _global_value = GLOBAL_VALUE.lock();
    // println!("global_value:{}",*_global_value);
    // 结束当前函数作用域时自动解锁
    _global_value.to_string()
}
async fn start_heart_rate() -> Result<(), Box<dyn Error>> {
    println!("start_heart_rate started");
    let adapter = Adapter::default()
        .await
        .ok_or("Bluetooth adapter not found")?;
    adapter.wait_available().await?;

    println!("starting scan");
    let mut scan = adapter.scan(&[]).await?;

    println!("scan started");
    while let Some(discovered_device) = scan.next().await {
        handle_device(discovered_device);
    }
    Ok(())
}

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}
#[tauri::command]
fn heartbeat() -> String {
    println!("{}", use_global_value());
    format!("{}", use_global_value())
}

#[tokio::main]
async fn main() {
    tauri::Builder::default()
        .setup(|_app| {
            tokio::spawn(async move {
                let result = start_heart_rate().await;
                match result {
                    Ok(_) => {
                        println!("Bluetooth scanner is running");
                    }
                    Err(err) => {
                        println!("Bluetooth scanner is not running");
                        println!("{}", err);
                    }
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![greet,heartbeat])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
