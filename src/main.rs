use directories::ProjectDirs;
use reqwest;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::fs;

#[tokio::main]
async fn main() {
    get_cache_dir_ready();
    let licenses = get_licenses().await;

    println!("We have the folloing licenses to choose from:");
    let mut index = 0;
    for license in licenses {
        println!("{index:>2}) {:<12} | {}", license.key, license.name);
        index = index + 1;
    }
}

fn get_cache_dir_ready() {
    let project_dir = ProjectDirs::from("com", "th3fishmk", "outspector").unwrap();
    let project_dir = project_dir.cache_dir();
    println!("{:?}", project_dir);
    std::fs::create_dir_all(project_dir).unwrap();
}

async fn get_licenses() -> Root {
    let project_dir = ProjectDirs::from("com", "th3fishmk", "outspector").unwrap();
    let project_dir = project_dir.cache_dir();

    let root_license_file = Path::new(project_dir).join("licenses");
    let content = fs::read_to_string(&root_license_file).await;

    match content {
        Err(_) => {
            println!("Getting remote licenses");
            get_remote_licenses(&root_license_file).await
        }
        Ok(x) => {
            println!("License file found, reusing licenses");
            serde_json::from_str(x.as_str()).unwrap()
        }
    }
}

async fn get_remote_licenses(save_file: &PathBuf) -> Root {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap();

    let data: Root = client
        .get("https://api.github.com/licenses")
        .header("User-Agent", "my-cli/1.0")
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
            println!("Error saving liceses to the file {:?}: {:?}", save_file, x)
        }
        Ok(()) => {
            println!("Licenses saved for future usage on: {:?}", save_file)
        }
    }

    data
}

pub type Root = Vec<Root2>;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Root2 {
    pub key: String,
    pub name: String,
    #[serde(rename = "spdx_id")]
    pub spdx_id: String,
    pub url: String,
    #[serde(rename = "node_id")]
    pub node_id: String,
}
