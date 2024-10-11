use clap::Parser;
use std::fs;
use std::io::{self, BufRead, BufReader, prelude::*};
use anyhow::Result;
use std::env;
use std::process::{Command, Stdio};

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

    let conf_path = if args.path.is_empty() {
        //String::from("./links")
        get_links_dir()
    } else {
        args.path.clone()
    };

    // let dmenu_output = run_dmenu(conf_path);
    set_links_file(conf_path);

    Ok(())
}


fn get_links_dir() -> String {
    match env::var("XDG_CONFIG_HOME") {
        Ok(v) => {
            String::from(v + "/twer/")
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

fn set_links_file(links_dir: String) {
    // Check if links_dir already exists.
    match fs::read_dir(&links_dir) {
        Ok(dirs) => {
            println!("{dirs:?}");
        },

        Err(_) => {
            match fs::create_dir(&links_dir) {
                Err(why) => {
                    println!("ERROR: {why}.");
                },
                Ok(_) => {
                    println!("Created directory {links_dir}.");
                },
            };
        },
    }
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
