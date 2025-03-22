use std::collections::HashMap;
use std::process::Command;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::Path;
use zip::ZipArchive;
use serde::Deserialize;
use std::os::unix::fs::PermissionsExt;

#[derive(Deserialize)]
struct PluginManifest {
    name: String,
    version: String,
    description: Option<String>,
    script: String,
}

#[derive(Clone)] // Добавляем Clone
pub struct PluginManager {
    plugins: HashMap<String, (String, Option<String>)>,
    plugin_dir: String,
}

impl PluginManager {
    pub fn new() -> Self {
        let plugin_dir = dirs::home_dir()
            .unwrap()
            .join(".duckshell/plugins")
            .to_string_lossy()
            .to_string();
        fs::create_dir_all(&plugin_dir).unwrap_or(());
        PluginManager {
            plugins: HashMap::new(),
            plugin_dir,
        }
    }

    pub fn register(&mut self, name: &str, path: &str) {
        self.plugins.insert(name.to_string(), (path.to_string(), None));
    }

    pub fn execute(&self, name: &str, args: &[&str]) {
        if let Some((path, _)) = self.plugins.get(name) {
            let output = Command::new(path)
                .args(args)
                .output();
            match output {
                Ok(result) => {
                    if result.status.success() {
                        println!("{}", String::from_utf8_lossy(&result.stdout));
                    } else {
                        println!("Error: {}", String::from_utf8_lossy(&result.stderr));
                    }
                }
                Err(_) => println!("Quack? Plugin failed: {}", name),
            }
        } else {
            println!("Quack? Plugin not found: {}", name);
        }
    }

    pub fn has_plugin(&self, name: &str) -> bool {
        self.plugins.contains_key(name)
    }

    pub fn install(&mut self, name: &str, path: &str, source: Option<String>) {
        self.plugins.insert(name.to_string(), (path.to_string(), source));
        println!("Quack! Installed plugin: {}", name);
    }

    pub fn install_from_pfds(&mut self, pfds_path: &str, source: Option<String>) -> io::Result<()> {
        let file = File::open(pfds_path)?;
        let mut archive = ZipArchive::new(file)?;

        let manifest: PluginManifest = {
            let mut manifest_file = archive.by_name("manifest.json")
                .map_err(|_| io::Error::new(io::ErrorKind::NotFound, "manifest.json not found"))?;
            let mut manifest_content = String::new();
            manifest_file.read_to_string(&mut manifest_content)?;
            serde_json::from_str(&manifest_content)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
        };

        let script_path = Path::new(&self.plugin_dir).join(&manifest.name);
        if let Ok(mut script_file) = archive.by_name(&manifest.script) {
            let mut script_content = Vec::new();
            script_file.read_to_end(&mut script_content)?;
            let mut dest_file = File::create(&script_path)?;
            dest_file.write_all(&script_content)?;
            fs::set_permissions(&script_path, fs::Permissions::from_mode(0o755))?;
        } else {
            fs::write(&script_path, "#!/bin/sh\necho 'Quack! Default script'")?;
            fs::set_permissions(&script_path, fs::Permissions::from_mode(0o755))?;
        }

        self.install(&manifest.name, script_path.to_str().unwrap(), source);
        Ok(())
    }

    pub fn remove(&mut self, name: &str) {
        if let Some((path, _)) = self.plugins.remove(name) {
            let _ = fs::remove_file(path);
            println!("Quack! Removed plugin: {}", name);
        } else {
            println!("Quack? Plugin not found: {}", name);
        }
    }

    pub fn list(&self) {
        if self.plugins.is_empty() {
            println!("Quack! No plugins installed.");
        } else {
            println!("🦆 Installed plugins:");
            for (name, (path, source)) in &self.plugins {
                match source {
                    Some(url) => println!("  {} -> {} (from: {})", name, path, url),
                    None => println!("  {} -> {}", name, path),
                }
            }
        }
    }

    pub fn update(&mut self) -> io::Result<()> {
        println!("Quack! Updating plugins...");
        let plugins = self.plugins.clone();
        for (name, (_, source)) in plugins {
            if let Some(url) = source {
                println!("Updating {} from {}", name, url);
                self.remove(&name);
                let pfds_path = dirs::home_dir()
                    .unwrap()
                    .join(".duckshell/plugins/temp.pfds")
                    .to_string_lossy()
                    .to_string();
                let response = reqwest::blocking::get(&url)
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
                let bytes = response.bytes()
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
                let mut file = File::create(&pfds_path)?;
                file.write_all(&bytes)?;
                self.install_from_pfds(&pfds_path, Some(url))?;
                fs::remove_file(&pfds_path)?;
            }
        }
        println!("Quack! All plugins updated.");
        Ok(())
    }

    pub fn get_plugin_names(&self) -> Vec<String> {
        self.plugins.keys().cloned().collect()
    }
}