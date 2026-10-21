use chrono::prelude::*;
use directories::ProjectDirs;
use reqwest;
use std::{
    env::{self},
    io::Write,
    path::{Path, PathBuf},
};
use tokio::fs;

use crate::inter::{License, Licenses};
mod inter;

#[tokio::main]
async fn main() {
    // get_cache_dir_ready();
    let licenses = get_licenses().await;

    // print and select license
    println!("We have the following licenses to choose from:");
    let mut index = 0;
    for license in &licenses {
        println!("{index:>2}) {:<12} | {}", license.key, license.name);
        index = index + 1;
    }
    let user_selection = ask_for_selection(licenses.iter().count());
    let id_to_add = licenses.iter().nth(user_selection);
    match id_to_add {
        None => {
            panic!(
                "No license with number: {}, please enter a number between 0-{}",
                user_selection,
                licenses.iter().count()
            );
        }
        Some(x) => {
            println!("Adding: {} {}", x.key, x.name);
            let target_license = get_license(&x.key).await;
            // println!("The license to add is:");
            // println!("{}", target_license.name);
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
    let mut current_dir = env::args().collect::<Vec<String>>()[0].to_string();
    if current_dir == "target/debug/outspector" {
        current_dir = ".".to_string();
    }
    println!("Adding license to: {current_dir}");
    let filename = PathBuf::from(current_dir).join("LICENSE");
    let file_exist = std::fs::exists(&filename);
    match file_exist {
        Err(x) => {
            panic!("{}", x)
        }
        Ok(x) => {
            if x == true {
                panic!("The file {:?} already exist!", x)
            } else {
                let saving = fs::write(filename, license_body).await;
                match saving {
                    Err(x) => {
                        println!("Error: {}", x)
                    }
                    Ok(()) => {
                        println!("License file successfully created!")
                    }
                }
            }
        }
    }
}

fn ask_for_selection(max: usize) -> usize {
    println!();
    print!("Select a license number (0-{}): ", max - 1);
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
    let project_dir = ProjectDirs::from("com", "th3fishmk", "outspector").unwrap();
    let project_dir = project_dir.cache_dir();
    let root_license_file = Path::new(project_dir).join("licenses");

    let content = fs::read_to_string(&root_license_file).await;
    match content {
        Err(_) => {
            std::fs::create_dir_all(project_dir).unwrap();
            // println!("Getting remote licenses");
            get_remote_licenses(&root_license_file).await
        }
        Ok(x) => {
            // println!("License file found, reusing licenses");
            let file_content = serde_json::from_str(x.as_str());
            match file_content {
                Err(_) => {
                    std::fs::create_dir_all(project_dir).unwrap();
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
        .unwrap();

    let data: Vec<Licenses> = client
        .get("https://api.github.com/licenses")
        .header("User-Agent", "outspector/1.0")
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2026-03-10")
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let content = serde_json::to_string(&data).unwrap();
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
    let project_dir = ProjectDirs::from("com", "th3fishmk", "outspector").unwrap();
    let project_dir = project_dir.cache_dir();
    let target_license = Path::new(project_dir).join(name);

    let license = get_local_license(&target_license).await;
    match license {
        None => {
            // println!("No local license, getting remote");
            let lic = get_remote_license(name, &target_license).await;
            lic
        }
        Some(x) => {
            let lic: License = serde_json::from_str(&x).unwrap();
            // println!("We have a local license: {}", lic.name);
            lic
        }
    }
}

async fn get_local_license(target: &PathBuf) -> Option<String> {
    let file_content = fs::read_to_string(target).await;
    match file_content {
        Err(_) => None,
        Ok(x) => Some(x),
    }
}

async fn get_remote_license(name: &str, save_file: &PathBuf) -> License {
    let url = format!("https://api.github.com/licenses/{name}");
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap();

    let data: License = client
        .get(url)
        .header("User-Agent", "outspector/1.0")
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2026-03-10")
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let content = serde_json::to_string(&data).unwrap();
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
