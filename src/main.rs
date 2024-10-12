use clap::Parser;
use std::fs;
use std::io::{self, BufRead, BufReader, prelude::*};
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::env;
use std::process::{Command, Stdio};
use std::collections::HashMap;

#[derive(Debug, clap::Parser)]
#[command(author, version)]
#[command(about = "Call streamlink with mpv", long_about = None)]

struct Args {
    #[arg(value_name = "URL", default_value_t = String::from(""))]
    url: String,

    #[arg(short = 's', long = "source", default_value_t = String::from("twitch"))]
    source: String,

    #[arg(short = 'p', long = "path", default_value_t = String::from("") )]
    path: String,

}

fn main() {
    let args = Args::parse();
    if let Err(e) = run(&args) {
        eprintln!("ERROR: {e}");
        std::process::exit(1);
    }
}

fn run(args: &Args) -> Result<()> {

    // let conf_path = if args.path.is_empty() {
    //     //String::from("./links")
    //     get_config_dir()
    // } else {
    //     args.path.clone()
    // };
    let conf_path = get_config_dir();

    //let dmenu_output = run_dmenu(conf_path);
    set_check_config(&conf_path);
    let config_map = parse_config(conf_path + "twer.conf");

    Ok(())
}


fn get_config_dir() -> String {
    match env::var("XDG_CONFIG_HOME") {
        Ok(xdg_config_dir) => {
            String::from(xdg_config_dir + "/twer/")
        },
        Err(_) => {
            match env::var("HOME") {
                Ok(home_dir) => {
                    String::from(home_dir + "/.local/etc/twer/")
                },
                Err(_) => {
                    panic!("Cannot find either $XDG_CONFIG_HOME or $HOME. Exiting.")
                },
            }
        },
    }
}

fn set_check_config(links_dir: &String) {
    let conf_path = PathBuf::from(links_dir.clone() + "twer.conf");
    let links_path = PathBuf::from(links_dir.clone() + "links");

    let conf_exists = fs::metadata(&conf_path).is_ok();
    let links_exists = fs::metadata(&links_path).is_ok();

    if conf_exists && links_exists {
        return;
    }

    // Create director
    match fs::read_dir(&links_dir) {
        Ok(_) => {
            if !conf_exists {
                match fs::File::create_new(conf_path) {
                    Err(why) => {
                        panic!("ERROR: Can not create config file. ({why})");
                    },
                    Ok(_) => {},
                };
            }
            if !links_exists {
                match fs::File::create_new(links_path) {
                    Err(why) => {
                        panic!("ERROR: Can not create config file. ({why})");
                    },
                    Ok(_) => {},
                };
            }
            return;
        },
        Err(_) => {
            match fs::create_dir(&links_dir) {
                Err(why) => {
                    panic!("ERROR: {why}.");
                },
                Ok(_) => {
                    println!("Created directory {links_dir}.");
                    set_check_config(links_dir); // Recurse to create files.
                    return;
                },
            }
        },
    }
}

fn parse_config(conf_path: String) -> HashMap<String, String> {
    let config = fs::read_to_string(conf_path).expect("Failed to read config file.");
    let mut config_map: HashMap<String, String> = HashMap::new();

    for line in config.lines() {
        let line = &line[..line.find('#').unwrap_or(line.len())];
        if line.trim().is_empty() {
            continue;
        }
        let mut key_val = line.splitn(2, '=');
        if let (Some(key), Some(val)) = (key_val.next(), key_val.next()) {
            let key = key.trim().to_string();
            let val = val.trim().to_string();
            config_map.insert(key, val);
        }
    }
    return config_map;
}

fn run_dmenu(conf_path: String) -> String {
    // Read content of the file into memory.
    let links = fs::read_to_string(conf_path).expect("Failed to read links file.");

    // Spawn dmenu, make it wait for input.
    let process = match Command::new("dmenu")
        .arg("-i")
        .arg("-l")
        .arg("10")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
    {
        Err(why) => panic!("Cannot spawn dmenu: {why}"),
        Ok(proc) => proc,
    };

    // Write the links file to dmenu's stdio. .unwrap() is fine because we requested
    // Stdio::piped(): stdin will always be ready since we got here without a panic.
    // It is probably better to use .expect() to be more verbose when error happens.
    match process.stdin.unwrap().write_all(links.as_bytes()) {
        Err(why) => panic!("Cannot write to dmenu's stdin: {why}"),
        Ok(_) => {},
    }

    // Catch the user's selection from dmenu.
    let mut stdout_str = String::new();
    match process.stdout.unwrap().read_to_string(&mut stdout_str) {
        Err(why) => panic!("Cannot read dmenu's stdout: {why}"),
        Ok(_) => stdout_str,
    }
}
