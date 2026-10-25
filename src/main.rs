use crate::interfaces::{License, Licenses};
use chrono::prelude::*;
use directories::ProjectDirs;
use std::{
    env::{self},
    io::Write,
    path::{Path, PathBuf},
};
use tokio::fs;
mod interfaces;

// Exit codes:
// 1 = IO error
// 2 = Argument errors

#[tokio::main]
async fn main() {
    let licenses = get_licenses().await;

    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        choose_license(licenses).await;
    } else if args.len() == 2 {
        let user_request = &args[1];
        println!("Looking for the following license: {}", args[1]);
        let target_license: Vec<_> = licenses.iter().filter(|f| f.key == *user_request).collect();
        if target_license.len() == 1 {
            println!(
                "Found the license: {} : {:?}",
                target_license[0].key, target_license[0].name
            );
            let license = get_license(&target_license[0].key).await;
            add_license_to_dir(&license).await;
        } else {
            eprintln!("License {} was not found", args[1]);
            std::process::exit(2)
        }
    } else {
        eprintln!("Invalid arguments!");
        std::process::exit(2)
    }
}

async fn choose_license(licenses: Vec<Licenses>) {
    println!("We have the following licenses to choose from:");
    println!("{:>2}  {:<12} | Name", "ID", "Key");
    for (index, license) in licenses.iter().enumerate() {
        println!("{index:>2}) {:<12} | {}", license.key, license.name);
    }
    let user_selection = ask_for_selection(licenses.len());
    let id_to_add = licenses.get(user_selection);
    match id_to_add {
        None => {
            panic!(
                "No license with number: {}, please enter a number between 0-{}",
                user_selection,
                licenses.len()
            );
        }
        Some(x) => {
            println!("Adding: {} {}", x.key, x.name);
            let target_license = get_license(&x.key).await;
            add_license_to_dir(&target_license).await;
        }
    }
}

async fn add_license_to_dir(license: &License) {
    let mut license_body = license.body.to_string();
    if &license.key == "mit" {
        let current_year = Utc::now().year().to_string();
        license_body = license_body.replace("[year]", &current_year);
        print!("This license require a name, please enter it now: ");
        let _ = std::io::stdout().flush();
        let mut name_from_input = String::new();
        std::io::stdin()
            .read_line(&mut name_from_input)
            .expect("Invalid input");
        name_from_input = name_from_input.trim().to_string();

        if name_from_input.is_empty() {
            println!("No name, please remember to change this field in the file.")
        } else {
            license_body = license_body.replace("[fullname]", &name_from_input);
        }
    }
    let current_dir = env::current_dir().expect("Error getting the current directory");
    let filename = PathBuf::from(&current_dir).join("LICENSE");
    let file_exist = std::fs::exists(&filename);
    match file_exist {
        Err(x) => {
            eprintln!("We encounter an error while trying to save the file: {}", x);
            std::process::exit(1)
        }
        Ok(x) => {
            if x {
                eprintln!("The file {:?} already exist!", filename);
                std::process::exit(1)
            } else {
                let saving = fs::write(filename, license_body).await;
                match saving {
                    Err(x) => {
                        eprintln!("Error: {}", x);
                        std::process::exit(1)
                    }
                    Ok(()) => println!("License file successfully created!"),
                }
            }
        }
    }
}

fn ask_for_selection(max: usize) -> usize {
    println!();
    print!("Select a license number(id) (0-{}): ", max - 1);
    let _ = std::io::stdout().flush();
    let mut selection = String::new();
    std::io::stdin()
        .read_line(&mut selection)
        .expect("Invalid input");
    selection = selection.trim().to_string();

    let selection = selection.parse();
    match selection {
        Err(x) => {
            panic!("Error parsing the input: {x}")
        }
        Ok(x) => x,
    }
}

async fn get_licenses() -> Vec<Licenses> {
    let project_dir = ProjectDirs::from("com", "th3fishmk", "outspector")
        .expect("No valid home directory path could be retrieved from the operating system.");
    let project_dir = project_dir.cache_dir();
    let root_license_file = Path::new(project_dir).join("licenses");
    let content = fs::read_to_string(&root_license_file).await;
    match content {
        Err(_) => {
            std::fs::create_dir_all(project_dir).expect("Error trying to create a cache dir.");
            // println!("Getting remote licenses");
            get_remote_licenses(&root_license_file).await
        }
        Ok(x) => {
            // println!("License file found, reusing licenses");
            let file_content = serde_json::from_str(x.as_str());
            match file_content {
                Err(_) => {
                    std::fs::create_dir_all(project_dir)
                        .expect("Error trying to create a cache dir.");
                    // println!("Error reading the file content, getting remote licenses");
                    get_remote_licenses(&root_license_file).await
                }
                Ok(x) => x,
            }
        }
    }
}

async fn get_remote_licenses(save_file: &PathBuf) -> Vec<Licenses> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .expect("Error trying to create a client");

    let data: Vec<Licenses> = client
        .get("https://api.github.com/licenses")
        .header("User-Agent", "outspector/1.0")
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2026-03-10")
        .send()
        .await
        .expect("Error getting data from the github api.")
        .json()
        .await
        .expect("Invalid json data.");
    let content = serde_json::to_string(&data).expect("Error parsing json data.");
    let save = fs::write(&save_file, content).await;
    match save {
        Err(x) => {
            println!("Error saving licenses to the file {:?}: {:?}", save_file, x)
        }
        Ok(()) => {
            // println!("Licenses saved for future usage on: {:?}", save_file)
        }
    }
    data
}

async fn get_license(name: &str) -> License {
    let project_dir = ProjectDirs::from("com", "th3fishmk", "outspector")
        .expect("No valid home directory path could be retrieved from the operating system.");
    let project_dir = project_dir.cache_dir();
    let target_license = Path::new(project_dir).join(name);

    let license = get_local_license(&target_license).await;
    match license {
        None => get_remote_license(name, &target_license).await,
        Some(x) => serde_json::from_str(&x).expect("Error parsing json data"),
    }
}

async fn get_local_license(target: &PathBuf) -> Option<String> {
    let file_content = fs::read_to_string(target).await;
    file_content.ok()
}

async fn get_remote_license(name: &str, save_file: &PathBuf) -> License {
    let url = format!("https://api.github.com/licenses/{name}");
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .expect("Error trying to create a client");

    let data: License = client
        .get(url)
        .header("User-Agent", "outspector/1.0")
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2026-03-10")
        .send()
        .await
        .expect("Error getting data from the github api.")
        .json()
        .await
        .expect("Invalid json data.");
    let content = serde_json::to_string(&data).expect("Error parsing json data.");
    let save = fs::write(&save_file, content).await;
    match save {
        Err(x) => {
            println!("Error saving licenses to the file {:?}: {:?}", save_file, x)
        }
        Ok(()) => {
            // println!("Licenses saved for future usage on: {:?}", save_file)
        }
    }
    data
}
