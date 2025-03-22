use rustyline::error::ReadlineError;
use rustyline::history::DefaultHistory;
use rustyline::Editor;
use std::process::Command;
use sysinfo::{System, CpuRefreshKind};

fn main() {
    println!("DuckShell v0.1.0 - Quack quack! 🦆");

    let mut rl = Editor::<(), DefaultHistory>::new()
        .expect("Failed to initialize readline");
    loop {
        let readline = rl.readline("🦆> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str()); 
                process_command(&line);
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                println!("Quack! Exiting DuckShell.");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
}

fn process_command(cmd: &str) {
    let parts: Vec<&str> = cmd.trim().split_whitespace().collect();
    if parts.is_empty() {
        return;
    }

    match parts[0] {
        "quack" => {
            if parts.len() > 1 {
                println!("Quack! You said: {}", &parts[1..].join(" "));
            } else {
                println!("Quack quack!");
            }
        }
        "dsh" => handle_dsh_command(&parts[1..]),
        "exit" => std::process::exit(0),
        _ => {
            let output = Command::new(parts[0])
                .args(&parts[1..])
                .output();

            match output {
                Ok(result) => {
                    if result.status.success() {
                        println!("{}", String::from_utf8_lossy(&result.stdout));
                    } else {
                        println!("Error: {}", String::from_utf8_lossy(&result.stderr));
                    }
                }
                Err(_) => println!("Quack? Unknown command: {}", cmd),
            }
        }
    }
}

fn handle_dsh_command(args: &[&str]) {
    if args.is_empty() {
        println!("Quack! Use 'dsh --version' or 'dsh --info'");
        return;
    }

    match args[0] {
        "--version" | "-v" => println!("DuckShell v0.1.0 - Quack quack!"),
        "--info" => display_system_info(),
        _ => println!("Quack? Unknown dsh option: {}", args[0]),
    }
}

fn display_system_info() {
    let mut sys = System::new_all();
    sys.refresh_cpu_specifics(CpuRefreshKind::everything()); 

    println!("🦆 DuckShell System Info");
    println!("Host: {}", System::host_name().unwrap_or("Unknown".to_string()));
    println!("CPU: {}", sys.global_cpu_info().brand());
    println!("RAM: {} MB", sys.total_memory() / 1024);
}