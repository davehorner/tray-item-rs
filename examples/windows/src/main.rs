 //#![windows_subsystem = "windows"]
 // uncomment to turn off the console for a taskbar only application

use std::ffi::OsStr;
use std::sync::{mpsc, Arc, Mutex};
use std::{thread, env};
use std::sync::atomic::{AtomicBool, Ordering};
use tray_item::{IconSource, TrayItem};

#[allow(dead_code)]
enum Message {
    Noise,
    Quit,
        TrayRightClicked,
TrayLeftClicked,
    ToggleState,
}
use std::io;
use winreg::enums::*;
use winreg::RegKey;
use std::os::windows::ffi::OsStrExt;
use winreg::RegValue;
use std::process::Command;
use dotenv::dotenv;
use std::collections::HashMap;

fn idle_loop(_rx: Arc<Mutex<mpsc::Receiver<Message>>>, control_flag: Arc<AtomicBool>) {
    const CHECK_INTERVAL_SECONDS: u64 = 3;

    let mut child_process: Option<std::process::Child> = None;

    while !control_flag.load(Ordering::Relaxed) {
        // Simulate some activity based on your requirements
        // Check for the "ToggleState" message

        // If the child process is not running, spawn it
        if child_process.is_none() {
            let cmd = "run.bat"; // Replace with your actual command script
            match Command::new(cmd).spawn() {
                Ok(child) => {
                    let pid = child.id();
                    println!("Spawned process with PID: {}", pid);
                    child_process = Some(child);
                }
                Err(err) => {
                    eprintln!("Failed to spawn process: {}", err);
                }
            }
        }

        // Simulate some activity based on your requirements

        // Sleep for the specified interval
        println!("idle_loop");
        std::thread::sleep(std::time::Duration::from_secs(CHECK_INTERVAL_SECONDS));
    }

    let mut cleanup: Option<std::process::Child> = None;
    run_cleanup(&mut cleanup);

    // If the child process is running, kill it
    if let Some(mut child) = child_process.take() {
        let pid = child.id();
        println!("Killing process with PID: {}", pid);
        if let Err(err) = child.kill() {
            eprintln!("Failed to kill process with PID {}: {}", pid, err);
        }
    }

    println!("idle_loop exit");
}

fn main() {
    if let Ok(exe_path) = env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            if let Err(err) = env::set_current_dir(exe_dir) {
                eprintln!("Error changing current directory: {}", err);
            }
        } else {
            eprintln!("Unable to determine executable directory");
        }
    } else {
        eprintln!("Unable to determine executable path");
    }

    dotenv().ok();
    let mut tray = TrayItem::new("rawwrrr", IconSource::Resource("tray-default"), Message::TrayLeftClicked as u32, Message::TrayRightClicked as u32).unwrap();

    let (tx, rx) = mpsc::sync_channel(1);
    let control_rx = Arc::new(Mutex::new(rx));

    let control_flag = Arc::new(AtomicBool::new(false));
    let args: Vec<String> = env::args().collect();

    if args.len() >= 2 {
        match args[1].as_str() {
            "true" => set_autostart_registry_entry(true),
            "false" => set_autostart_registry_entry(false),
            _ => {
                println!("Invalid argument. Use 'true' or 'false'.");
                return;
            }
        };
    }

    let mut run_cmd_at_start: bool = env::var("RUN_CMD_AT_START")
        .unwrap_or_else(|_| "false".to_string())
        .parse()
        .unwrap_or(false);

    if args.len() == 3 {
        // Read the content of the .env file
        let env_file_path =format!(".env");
        let mut env_vars: HashMap<String, String> = HashMap::<String,String>::new();
        if std::path::Path::new(&env_file_path).exists() {
            let file_content = std::fs::read_to_string(&env_file_path).expect("Error reading .env file");
            env_vars = parse_env_file(&file_content);
        }

        match args[2].as_str() {
            "true" =>  { run_cmd_at_start=true; env_vars.insert("RUN_CMD_AT_START".to_string(), "true".to_string());  },
            "false" => { run_cmd_at_start=false; env_vars.remove("RUN_CMD_AT_START"); },
            _ => {
                println!("Invalid argument. Use 'true' or 'false'.");
                return;
            }
        };
        // Write the updated environment variables to the .env file
        std::fs::write(&env_file_path, serialize_env_file(&env_vars))
        .expect("Error writing to .env file");    
        println!("Run Command at Start: {}", run_cmd_at_start);
    }

    tray.add_label("quit").ok();
    // let red_tx = tx.clone();
    // tray.add_menu_item("Red", move || {
    //     red_tx.send(Message::ToggleState).unwrap();
    // }).unwrap();

    // let green_tx = tx.clone();
    // tray.add_menu_item("Green", move || {
    //     green_tx.send(Message::ToggleState).unwrap();
    // }).unwrap();

    let quit_tx = tx.clone();
    tray.add_menu_item("Quit", move || {
        quit_tx.send(Message::Quit).unwrap();
    }).unwrap();
    let tray_rx = tx.clone();
    tray.add_menu_item("Click", move || {
        tray_rx.send(Message::TrayRightClicked).unwrap();
    }).unwrap();
    let tray_rrx = tx.clone();
    tray.add_menu_item("Click", move || {
        tray_rrx.send(Message::TrayLeftClicked).unwrap();
    }).unwrap();

    let mut idle_handle: Option<thread::JoinHandle<()>> = None;
    let mut icon_color = String::from("Red"); // Start in the "Red" state
    // let autostart_enabled = read_autostart_registry_entry();
    if run_cmd_at_start { //autostart_enabled {
        fun_name(&mut icon_color, &mut tray, &control_flag, &control_rx, &mut idle_handle);
    }

    loop {
        while let Ok(message) = control_rx.lock().unwrap().try_recv() {
            match message {
                Message::Noise => {
                    println!("Noise {}", message as u32);
                },
                Message::TrayLeftClicked => {
                    println!("Click {}", message as u32);
                    fun_name(&mut icon_color, &mut tray, &control_flag, &control_rx, &mut idle_handle);
                }
                Message::TrayRightClicked => {
                    println!("RClick {}", message as u32);
                    perform_cleanup(&control_flag, &mut idle_handle);
                    return;
                }
                Message::Quit => {
                    println!("Quit");
                    perform_cleanup(&control_flag, &mut idle_handle);
                    return;
                }
                Message::ToggleState => {
                    fun_name(&mut icon_color, &mut tray, &control_flag, &control_rx, &mut idle_handle);
                }
            }
        }
    }
}

fn perform_cleanup(control_flag: &Arc<AtomicBool>, idle_handle: &mut Option<thread::JoinHandle<()>>) {
    control_flag.store(true, Ordering::Relaxed);

    // If there's an idle loop, wait for it to finish
    if let Some(handle) = idle_handle.take() {
        handle.join().unwrap();
    }
    let mut cleanup: Option<std::process::Child> = None;
    run_cleanup(&mut cleanup);
    if let Some(mut handle) = cleanup {
        handle.wait().ok();
    }
}

fn fun_name(icon_color: &mut String, tray: &mut TrayItem, control_flag: &Arc<AtomicBool>, control_rx: &Arc<Mutex<mpsc::Receiver<Message>>>, idle_handle: &mut Option<thread::JoinHandle<()>>) {
    if icon_color == "Red" {
        println!("Green");
        tray.set_icon(IconSource::Resource("name-of-icon-in-rc-file")).unwrap();
        *icon_color = String::from("Green");

        // Start the idle loop when transitioning to "Green"
        control_flag.store(false, Ordering::Relaxed);
        let control_rx = Arc::clone(control_rx);
        let control_flag = Arc::clone(control_flag);
        *idle_handle = Some(thread::spawn(move || {
            idle_loop(control_rx, control_flag);
        }));
    } else {
        println!("Red");
        tray.set_icon(IconSource::Resource("another-name-from-rc-file")).unwrap();
        *icon_color = String::from("Red");

        // If transitioning to "Red," set the control flag to true
        control_flag.store(true, Ordering::Relaxed);

        // If there's an idle loop, wait for it to finish
        if let Some(handle) = idle_handle.take() {
            handle.join().unwrap();
        }
    }
}

#[allow(dead_code)]
fn read_autostart_registry_entry() -> bool {
    // Specify the registry key path and value name
    let key_path = r"Software\Microsoft\Windows\CurrentVersion\Run";
    let value_name = "rawwrrr";

    // Open the registry key for reading
    let hklm = RegKey::predef(HKEY_CURRENT_USER);
    let key = hklm.open_subkey_with_flags(key_path, KEY_READ).unwrap();

    // Check if the value exists and read its content
    match key.get_value::<String, _>(value_name) {
        Ok(value) => {
            println!("Autostart value found: {}", value);
            // Add logic here based on the autostart value
            // For example, return true if autostart is enabled
            true
        }
        Err(ref e) if e.kind() == io::ErrorKind::NotFound => {
            println!("Autostart value not found");
            // Add logic here based on the absence of the autostart value
            // For example, return false if autostart is not enabled
            false
        }
        Err(e) => {
            // Handle other errors
            eprintln!("Error reading autostart registry entry: {}", e);
            // Add appropriate error handling logic here
            false
        }
    }
}

fn set_autostart_registry_entry(enable: bool) {
    // Specify the registry key path and value name
    let key_path = r"Software\Microsoft\Windows\CurrentVersion\Run";
    let value_name = "rawwrrr";

    // Open the registry key for writing
    let hklm = RegKey::predef(HKEY_CURRENT_USER);
    let key = hklm.create_subkey_with_flags(key_path, KEY_SET_VALUE).unwrap();

    if enable {
          // Set the autostart value to the path of your application executable
          let exe_path = std::env::current_exe().unwrap();

          // Convert u16 elements to u8 before collecting into a Vec<u8>
          let bytes: Vec<u8> = OsStr::new(exe_path.to_str().unwrap())
              .encode_wide()
              .flat_map(|c| [(c & 0xFF) as u8, ((c >> 8) & 0xFF) as u8].into_iter())
              .chain(Some(0))
              .collect();
        // Create a RegValue from the Vec<u8>
        let reg_value = RegValue {
            bytes,
            vtype: REG_SZ,
        };

        // Set the value in the registry
        key.0.set_raw_value(value_name, &reg_value).unwrap();

          println!("Autostart set to true");
    } else {
        // If you want to disable autostart, you can delete the registry entry
        match key.0.delete_value(value_name) {
            Ok(()) => println!("Autostart set to false"),
            Err(ref e) if e.kind() == io::ErrorKind::NotFound => {
                println!("Autostart value not found (already disabled)");
            }
            Err(e) => {
                eprintln!("Error deleting autostart registry entry: {}", e);
                // Add appropriate error handling logic here
            }
        }
    }
}

fn run_cleanup(child_process: &mut Option<std::process::Child>) {
    let cmd = "cleanup.bat";
    // Replace with your actual command script
    match Command::new(cmd).spawn() {
        Ok(child) => {
            let pid = child.id();
            println!("Spawned cleanup process with PID: {}", pid);
            *child_process = Some(child);
        }
        Err(err) => {
            eprintln!("Failed to spawn cleanup process: {}", err);
        }
    }
}

fn parse_env_file(file_content: &str) -> HashMap<String, String> {
    file_content
        .lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.splitn(2, '=').collect();
            if parts.len() == 2 {
                Some((parts[0].trim().to_string(), parts[1].trim().to_string()))
            } else {
                None
            }
        })
        .collect()
}

fn serialize_env_file(env_vars: &HashMap<String, String>) -> String {
    env_vars
        .iter()
        .map(|(key, value)| format!("{}={}", key, value))
        .collect::<Vec<_>>()
        .join("\n")
}

