// extern crate reqwest;
extern crate reqwest;
extern crate serde;
extern crate serde_json;

use std::env;
use tokio;


#[derive(Default, Debug, Clone, PartialEq, serde_derive::Serialize, serde_derive::Deserialize)]
pub struct Repo {
    pub name: String,
    pub path: String,
    pub sha: String,
    pub size: u64,
    pub url: String,
    pub html_url: String,
    pub git_url: String,
    pub download_url: Option<String>,
    #[serde(rename = "type")]
    pub type_field: String,
    #[serde(rename = "_links")]
    pub links: Links,
}

#[derive(Default, Debug, Clone, PartialEq, serde_derive::Serialize, serde_derive::Deserialize)]
pub struct Links {
    #[serde(rename = "self")]
    pub self_field: String,
    pub git: String,
    pub html: String,
}

async fn request_gitignore() -> Result<Vec<Repo>, reqwest::Error> {
    let client = reqwest::Client::new();

    let repos = client
        .get("https://api.github.com/repos/github/gitignore/contents")
        .header("User-Agent", "abnerfs")
        .send()
        .await?
        .json::<Vec<Repo>>()
        .await
        .unwrap();

    Ok(repos)
}

async fn download_file(url: &str, path: &str, file_name: &str) {

    let response = reqwest::get(url).await.expect("Failed to send download request");

    let get_file_name = | | {
        response
            .url()
            .path_segments()
            .and_then(|segments| segments.last())
            .and_then(|name| if name.is_empty() { None } else { Some(name) })
            .expect("Error while getting file name")
    };

    let fname = if file_name.is_empty() { get_file_name() } else { file_name };
    let full_path = format!("{}{}", path, fname);
    // println!("File will be located at {}", full_path);

    let mut file = tokio::fs::File::create(full_path).await.expect("Failed to create file");
    let bytes = response.bytes().await.expect("Error getting file bytes");
    tokio::io::copy(&mut &*bytes, &mut file).await.expect("Failed to download file");
}

async fn list_gitignore() -> Result<Vec<(String, String)>, reqwest::Error> {
    let repos = request_gitignore().await?;
    let repos = repos
        .iter()
        .map(|repo| {
            let download_url = match &repo.download_url {
                Some(value) => String::from(value),
                None => String::from(""),
            };
            (String::from(&repo.name), download_url)
        })
        .collect();

    Ok(repos)
}

#[tokio::main]
async fn main() {
    let args_list: Vec<String> = env::args().collect();

    if args_list.len() < 2 {
        println!("gitignore-downloader <file> (see file list at: https://github.com/github/gitignore)");
        std::process::exit(0);
    }

    let gitignore_file = String::from(&args_list[1]);

    match list_gitignore().await {
        Ok(files) => {
            let find_file = files.into_iter().find(|(name, _)| {
                let gitignore_replaced = gitignore_file.to_uppercase().replace(".GITIGNORE", "");
                let gitignore_replaced = format!("{}.gitignore", gitignore_replaced);

                name.to_uppercase() == gitignore_replaced.to_uppercase()
            });
            match find_file {
                Some((_, download_url)) => {
                    let download_path = "./";
                    println!("Downloading file {}", download_url);
                    download_file(&download_url, download_path, ".gitignore").await;
                    println!("File downloaded!");
                }
                None => {
                    println!("Invalid file {}, (see file list at: https://github.com/github/gitignore)", gitignore_file);
                    std::process::exit(0);
                }
            }
        }
        Err(err) => println!("Err: {:?}", err),
    }
}
