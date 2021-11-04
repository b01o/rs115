use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use rs115::functions::*;
fn app() -> i32 {
    let mut rt = Runtime::new();

    use clap::{load_yaml, App};
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    // clean subcommand
    if let Some(_) = matches.subcommand_matches("clean") {
        if rt.clean().is_err() {
            eprintln!("clean failed, loggin info not found or unable to delete...");
            return 1;
        }
    }

    // set-cookies subcommand
    if let Some(matches) = matches.subcommand_matches("set-cookies") {
        let cookies = matches.value_of("cookies").unwrap();
        if rt.set_cookies(cookies).is_err() {
            eprintln!("set_cookies failed, program was not able to write to files ");
            return 1;
        }
    }

    // check subcommand
    if let Some(matches) = matches.subcommand_matches("check") {
        if let Some(matched_str) = matches.value_of("list-of-names") {
            let file = matched_str;

            let mut forbiden_list: Option<File> = None;
            let mut failed_list: Option<File> = None;

            if let Some(path) = matches.value_of("output_forbiden_list") {
                let fpath = Path::new(path);
                if fpath.exists() {
                    eprintln!("file already exist: {}", path);
                    return 1;
                }
                if let Ok(file) = File::create(fpath) {
                    forbiden_list = Some(file);
                } else {
                    eprintln!("fail to create file: {}", path);
                    return 1;
                }
            }

            if let Some(path) = matches.value_of("output_failed_case") {
                let fpath = Path::new(path);
                if fpath.exists() {
                    eprintln!("file already exist: {}", path);
                    return 1;
                }
                if let Ok(file) = File::create(fpath) {
                    failed_list = Some(file);
                } else {
                    eprintln!("fail to create file: {}", path);
                    return 1;
                }
            }

            let file = File::open(file).unwrap();
            let file = BufReader::new(file);
            if rt
                .check_name_bulk_to_file(file, forbiden_list, failed_list)
                .is_err()
            {
                return 1;
            }
        } else {
            let name = matches.value_of("name").unwrap();
            if let Ok(is_valid) = rt.check_name(name) {
                if is_valid {
                    println!("name is VALID");
                    return 0;
                } else {
                    println!("name is NOT valid");
                    return 2;
                }
            } else {
                eprintln!("fail to check {}", name);
                return 1;
            }
        }
    }

    // status
    if let Some(matches) = matches.subcommand_matches("status") {
        if matches.is_present("cookies") {
            if rt.has_cookies() {
                println!("{}", rt.print_cookies());
            } else {
                println!("cookies not set!");
            }
        } else if matches.is_present("session") {
            println!("{:#?}", rt);
        } else {
            if rt.has_cookies() {
                println!("Normal");
            } else {
                println!("Warning: cookies not set!");
            }
        }
    }
    0
}

fn main() {
    let exit_code = app();
    std::process::exit(exit_code);
}
