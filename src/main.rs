use std::{fs::{self, File}, io, process::{self, Command}};

use reqwest::blocking::Response;

fn _fatal_message(message: &str) -> String {
    format!("\x1b[1;31m[Fatal]\x1b[0m {}", message)
}

fn exit_fatal(error_message: String, exit_code: i32) -> ! {
    println!("\x1b[1;31m[Fatal]\x1b[0m {}", error_message);
    process::exit(exit_code);
}

fn info(message: String) {
    println!("\x1b[1;34m[Info]\x1b[0m {}", message);
}

fn file_dir() -> String {
    String::from("/etc/zapretfiles")
}

/// Downloads files and save converted files
fn download() {
    const URL: &str = "https://raw.githubusercontent.com/Flowseal/zapret-discord-youtube/refs/heads/main/";
    // Files to download
    const DOWNLOAD_LIST: [&str; 10] = [
        "lists/ipset-all.txt",
        "lists/ipset-exclude.txt",
        "lists/list-exclude.txt",
        "lists/list-general.txt",
        "lists/list-google.txt",

        "bin/quic_initial_www_google_com.bin",
        "bin/tls_clienthello_4pda_to.bin",
        "bin/tls_clienthello_max_ru.bin",
        "bin/tls_clienthello_www_google_com.bin",
        "bin/stun.bin"
    ];
    const BATLIST: [&str; 19] = [
        "general.bat",
        "general (ALT).bat",
        "general (ALT2).bat",
        "general (ALT3).bat",
        "general (ALT4).bat",
        "general (ALT5).bat",
        "general (ALT6).bat",
        "general (ALT7).bat",
        "general (ALT8).bat",
        "general (ALT9).bat",
        "general (ALT10).bat",
        "general (ALT11).bat",
        "general (FAKE TLS AUTO).bat",
        "general (FAKE TLS AUTO ALT).bat",
        "general (FAKE TLS AUTO ALT2).bat",
        "general (FAKE TLS AUTO ALT3).bat",
        "general (SIMPLE FAKE).bat",
        "general (SIMPLE FAKE ALT).bat",
        "general (SIMPLE FAKE ALT2).bat",
    ];

    let home = file_dir();

    // Create directories
    let _ = fs::create_dir_all(format!("{}/lists", home));
    let _ = fs::create_dir_all(format!("{}/bin", home));

    // Downloading files
    for download in DOWNLOAD_LIST {
        let file: String = format!("{}/{}", home, download);
        info(format!("Downloading: {}", file));

        // Get response from server
        let target: String = format!("{}{}", URL, download);
        let mut response: Response = reqwest::blocking::get(target).expect(_fatal_message("Request error").as_str());

        // Saving file
        let mut file = File::create(file).expect(_fatal_message("Failed to create file").as_str());
        io::copy(&mut response, &mut file).expect(_fatal_message("Failed to copy contents").as_str());
    }
    info(String::from("Download finished\n"));

    // Getting .bat files, and save generated .nix
    for bat in BATLIST {
        let file: String = format!("{}/{}", home, bat.replace(".bat", ".nix"));
        info(format!("Converting: {}", file));

        // Get response from server
        let target: String = format!("{}{}", URL, bat);
        let response: Response = reqwest::blocking::get(target).expect(_fatal_message("Request error").as_str());

        // Convert it and save as file
        let generated: String = convert(get_options(response.text().unwrap_or_default()));
        let _ = fs::write(file, generated);
    }
    info(String::from("Converting finished\n"));
}

/// Get start options from .bat
fn get_options(content: String) -> Vec<String> {
    // Get only lines containing filters
    let mut params: Vec<String> = vec![];
    let mut is_params: bool = false;
    for line in content.lines() {
        // Check when "params" starts
        let mut line_splitted = line.split_whitespace();
        if line_splitted.next().unwrap_or("") == "start" {
            is_params = true;
        }

        if is_params {
            let mut target_line = line.to_owned();
            // delete " ^" from every line if needed
            if target_line.chars().last().unwrap_or('^') == '^' {
                target_line.pop();
            target_line.pop();
            }
            params.push(target_line);
        }

        // Check if next line is also "params"
        is_params = false;
        let last: &str = line_splitted.last().unwrap_or("");
        if last == "^" {
            is_params = true;
        }
    }
    params
}

/// Converts original .bat to new .nix
fn convert(options: Vec<String>) -> String {
    const GAME_FILTER: &str = "1024-65535";
    const UDP_PORTS: &str = "\"443\" \"1024:65535\"";
    let bin: String = format!("{}/bin/", file_dir());
    let lists: String = format!("{}/lists/", file_dir());

    let mut new_options: Vec<String> = options;
    let _ = new_options.remove(0);
    new_options = new_options
        .iter()
        .map(|s| s.replace("%GameFilter%", GAME_FILTER))
        .map(|s| s.replace("%BIN%", bin.as_str()))
        .map(|s| s.replace("%LISTS%", lists.as_str()))
        .map(|s| s.replace("\"", "\\\""))
        .map(|s| format!("\"{}\"", s))
        .collect();

    let res: String = format!(
        "{{ config, pkgs, ...}}:\n\n{{\n  services.zapret = {{\n    enable = true;\n    udpSupport = true;\n    udpPorts = [ {} ];\n    \n    params = [\n      {}\n    ];\n  }};\n}}",
        UDP_PORTS,
        new_options.join("\n\n")
    );

    res
}

/// Copy configurations to /etc/nixos/zapret
fn copy_files() {
    info(String::from("Copying files"));

    // Make /etc/nixos/zapret if it not exist
    let _ = Command::new("sudo")
        .arg("mkdir")
        .arg("/etc/nixos/zapret")
        .status()
        .expect(_fatal_message("Can't make /etc/nixos/zapret directory").as_str());

    // Copies all .nix files to /etc/nixos/zapret
    for obj in fs::read_dir(format!("{}/", file_dir())).expect(_fatal_message("Can't read directory").as_str()) {
        // Get object path and info it
        let obj = obj.expect(_fatal_message("Can't resolve object").as_str()).path();
        info(format!("Copying: {:?}", obj));

        if obj.is_file() {
            let _ = Command::new("sudo")
                .arg("cp")
                .arg(obj)
                .arg("/etc/nixos/zapret")
                .status().expect(_fatal_message("Can't copy file").as_str());
        }
    }
}

use nix::unistd::Uid;
fn main() {
    if !Uid::effective().is_root() {
        exit_fatal(String::from("Sudo is required"), 1);
    }
    let usage: String = String::from("USAGE: zapret-wintonixos-tool [OPTIONS]\nOptions:\n\t-nd\t No download (skip download stage)\n\t-nc\t No copy (don't copy generated .nix files to /etc/nixos/zapret)\n\nDo not use -nd and -nc at the same time because that's all program does");
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut do_download: bool = true;
    let mut do_copy = true;

    for arg in args {
        match arg.as_str() {
            "-nd" => do_download = false,
            "-nc" => do_copy = false,
            _ => exit_fatal(format!("Unknown argument: {}\n{}", arg, usage), 1)
        }
    }

    // Exit if everything is disabled
    if !do_download & !do_copy {
        exit_fatal(format!("Incorrect usage (check last usage line)\n{}", usage), 1);
    }

    if do_download {
        download();
    } else {
        info(String::from("Download skipped"));
    }
    if do_copy {
        copy_files();
    } else {
        info(String::from("Copy skipped"));
    }
}
