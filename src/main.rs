mod plugins;

use plugins::PluginManager;
use rustyline::error::ReadlineError;
use rustyline::history::DefaultHistory;
use rustyline::{Editor, CompletionType, Config};
use rustyline::completion::Completer;
use rustyline::validate::Validator;
use rustyline::highlight::Highlighter; // Добавляем Highlighter
use rustyline::hint::Hinter; // Исправляем Hint на Hinter
use rustyline::Helper;
use std::process::Command;
use sysinfo::{System, CpuRefreshKind};
use reqwest::blocking::get;
use std::fs::{self, File}; // Добавляем File
use std::io::{self, Write};

#[derive(Clone)]
struct DuckCompleter {
    plugins: PluginManager,
}

impl Helper for DuckCompleter {}

impl Completer for DuckCompleter {
    type Candidate = String;

    fn complete(&self, line: &str, _pos: usize, _ctx: &rustyline::Context<'_>) -> rustyline::Result<(usize, Vec<String>)> {
        let commands = vec!["quack", "dsh", "dupi", "exit"];
        let plugin_names = self.plugins.get_plugin_names();
        let all_completions: Vec<String> = commands.into_iter().map(String::from).chain(plugin_names).collect();

        let matches: Vec<String> = all_completions
            .into_iter()
            .filter(|cmd| cmd.starts_with(line))
            .collect();
        Ok((0, matches))
    }
}

impl Hinter for DuckCompleter {
    type Hint = String;

    fn hint(&self, line: &str, _pos: usize, _ctx: &rustyline::Context<'_>) -> Option<String> {
        let commands = vec!["quack", "dsh", "dupi", "exit"];
        let plugin_names = self.plugins.get_plugin_names();
        let all_completions: Vec<String> = commands.into_iter().map(String::from).chain(plugin_names).collect();

        if line.is_empty() {
            return None;
        }

        for cmd in &all_completions {
            if cmd.starts_with(line) {
                return Some(cmd[line.len()..].to_string());
            }
        }
        None
    }
}

impl Highlighter for DuckCompleter {} // Пустая реализация, если подсветка не нужна

impl Validator for DuckCompleter {}

fn main() {
    println!("DuckShell v0.1.0 - Quack quack! 🦆");

    let config = Config::builder()
        .completion_type(CompletionType::List)
        .build();
    let mut rl = Editor::<DuckCompleter, DefaultHistory>::with_config(config)
        .expect("Failed to initialize readline");
    let mut plugin_manager = PluginManager::new();
    rl.set_helper(Some(DuckCompleter { plugins: plugin_manager.clone() }));

    plugin_manager.register("test", "echo");

    loop {
        let readline = rl.readline("🦆> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                process_command(&line, &mut plugin_manager);
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

fn process_command(cmd: &str, plugins: &mut PluginManager) {
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
        "dupi" => handle_dupi_command(&parts[1..], plugins),
        "exit" => std::process::exit(0),
        _ => {
            if plugins.has_plugin(parts[0]) {
                plugins.execute(parts[0], &parts[1..]);
            } else {
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

fn handle_dupi_command(args: &[&str], plugins: &mut PluginManager) {
    if args.is_empty() {
        println!("Quack! Use 'dupi -i <plugin|.pfds>', 'dupi -re <plugin>', 'dupi -ls', 'dupi -ud', or 'dupi -d <url>'");
        return;
    }

    match args[0] {
        "-i" => {
            if args.len() < 2 {
                println!("Quack? Specify a plugin name or .pfds file: 'dupi -i <plugin|.pfds>'");
            } else {
                if args[1].ends_with(".pfds") {
                    match plugins.install_from_pfds(args[1], None) {
                        Ok(()) => {}
                        Err(e) => println!("Quack? Failed to install .pfds: {}", e),
                    }
                } else {
                    plugins.install(args[1], args[1], None);
                }
            }
        }
        "-re" => {
            if args.len() < 2 {
                println!("Quack? Specify a plugin name: 'dupi -re <plugin>'");
            } else {
                plugins.remove(args[1]);
            }
        }
        "-ls" => plugins.list(),
        "-ud" => {
            match plugins.update() {
                Ok(()) => {}
                Err(e) => println!("Quack? Update failed: {}", e),
            }
        }
        "-d" => {
            if args.len() < 2 {
                println!("Quack? Specify a URL: 'dupi -d <url>'");
            } else {
                match download_plugin(args[1], plugins) {
                    Ok(()) => {}
                    Err(e) => println!("Quack? Failed to download plugin: {}", e),
                }
            }
        }
        _ => println!("Quack? Unknown dupi option: {}", args[0]),
    }
}

fn download_plugin(url: &str, plugins: &mut PluginManager) -> io::Result<()> {
    let response = get(url).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    let bytes = response.bytes().map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    let pfds_path = dirs::home_dir()
        .unwrap()
        .join(".duckshell/plugins/temp.pfds")
        .to_string_lossy()
        .to_string();
    let mut file = File::create(&pfds_path)?;
    file.write_all(&bytes)?;
    plugins.install_from_pfds(&pfds_path, Some(url.to_string()))?;
    fs::remove_file(&pfds_path)?;
    Ok(())
}

fn display_system_info() {
    let mut sys = System::new_all();
    sys.refresh_cpu_specifics(CpuRefreshKind::everything());
    println!("🦆 DuckShell System Info");
    println!("Host: {}", System::host_name().unwrap_or("Unknown".to_string()));
    println!("CPU: {}", sys.global_cpu_info().brand());
    println!("RAM: {} MB", sys.total_memory() / 1024);
}