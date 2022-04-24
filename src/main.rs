use std::path::PathBuf;

use reqwest::{self, header::HeaderMap};
use reqwest::header::*;

use serde::Deserialize;

use serde_json;

use std::collections::HashMap;
use std::fs;
use std::io::prelude::*;
use std::io::BufReader;

use base64;


use clap::Parser;

/// Migrate git repositories to a Gitea instance
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// (default) migrate only public repos
    #[clap(long)]
    public: bool,

    /// migrate only private repos
    #[clap(long)]
    private: bool,

    /// migrate only public and private repos
    #[clap(long)]
    both: bool,

    /// migrate all repos (including associated repos)
    #[clap(long)]
    all: bool,

    /// base url of github source repo
    #[clap(short, long, value_name = "URL")]
    from_url: String,

    /// base url of gitea destination repo
    #[clap(short, long, value_name = "URL")]
    to_url: String,

    
    /// print useful information to stdout
    #[clap(short, long)]
    verbose: bool,
    
    /// (optional) uses a custom credentials file with:
    /// github_user:github_token gitea_user:gitea_password
    #[clap(short, long, parse(from_os_str), value_name = "FILE")]
    creds: Option<PathBuf>,
}



// #[derive(Debug)]
// struct Repo {
//     name: String,
//     visibility: String,
//     owner: String,
// }

// fn get_token(path: &str) -> String {

//     let inner = match fs::File::open(path) {
//         Ok(file) => file,
//         Err(e) => panic!("{:?}", e),
//     };
    
//     let mut reader = BufReader::new(inner);
//     let mut buf: String = String::new();
//     reader.read_line(&mut buf).expect("Unable to read line");

//     buf
// }

// fn github_headers() -> HeaderMap {


//     let mut headers = HeaderMap::new();
//     let username = "maxgallup";
//     let password = get_token("token-github.txt");
    
//     let encoded_credentials = base64::encode(format!("{}:{}", username, password));
//     let basic_auth = format!("Basic {}", encoded_credentials);
    
//     headers.insert(AUTHORIZATION, HeaderValue::from_str(&basic_auth).unwrap());
//     headers.insert(USER_AGENT, HeaderValue::from_static("api"));
//     headers.insert(ACCEPT, HeaderValue::from_static("application/vnd.github.v3+json"));
//     headers
// }

// fn gitea_headers() -> HeaderMap {
//     let mut headers = HeaderMap::new();
//     headers.insert(USER_AGENT, HeaderValue::from_static("api"));
//     headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
//     headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
//     headers
// }

// fn trim_quotes(v: &serde_json::Value) -> String {
//     v.to_string().trim_matches('\"').to_string()
// }

// fn get_content(r: &Repo) -> HashMap<String, String> {



//     HashMap::new()
// }


// fn migrate(r: &Repo) {
//     gitea_client.post("ads").json(get_content(&repo)).send()?;
// }

#[derive(Debug)]
struct RepoFilter {
    public: bool,
    private: bool,
    any_owner: bool,
}

#[derive(Debug)]
struct UserData {
    gh: String,
    gt: String,
    gh_pass: String,
    gt_pass: String,
}

fn creds_from_file(path: PathBuf) -> Option<UserData> {

    let inner = match fs::File::open(path) {
        Ok(file) => file,
        Err(e) => panic!("{:?}", e),
    };

    let mut reader = BufReader::new(inner);
    let mut github: String = String::new();
    let mut gitea: String = String::new();
    reader.read_line(&mut github).expect("Unable to read line");
    reader.read_line(&mut gitea).expect("Unable to read line");

    let github : Vec<&str> = github.trim().split(":").collect();
    let gitea : Vec<&str> = gitea.trim().split(":").collect();

    Some(UserData {
        gh: github[0].to_string(),
        gt: gitea[0].to_string(),
        gh_pass: github[1].to_string(),
        gt_pass: gitea[1].to_string(),
    })
}

fn ask_for_creds() -> Option<UserData> {
    Some(UserData {
        gh: "ask".to_string(),
        gt: "ask".to_string(),
        gh_pass: "ask".to_string(),
        gt_pass: "ask".to_string(),
    })
}

fn main() -> Result<(), reqwest::Error> {

    let args = Args::parse();

    let gh_url = args.from_url;
    let gt_url = args.to_url;

    let user : Option<UserData> = match args.creds {
        Some(path) => creds_from_file(path),
        None => ask_for_creds(),
    };

    let filter = RepoFilter {
        public: !args.private || args.both || args.all,
        private: args.private || args.both || args.all,
        any_owner: args.all,
    };

    println!("{:?}", user);

    // let user = "maxgallup";
    // let github_base = "https://api.github.com/";
    // let gitea_base = "https://git.basingse.org/api/v1";
    // let all_repos = "https://api.github.com/user/repos?per_page=200";
    // let public_repos = format!("{}users/{}/repos", github_base, user);

    // let mut gh_db : Vec<Repo> = Vec::new();
  
    
    // let gh_client = reqwest::blocking::Client::builder()
    //     .default_headers(github_headers())
    //     .build()?;


    // let resp = gh_client.get(public_repos).send()?;


    // // println!("resp.status = {:?}", resp.status());
    // // println!("resp.headers = {:?}", resp.headers());
    // // println!("resp.text = {:?}", resp.text()?);
    // let body : serde_json::Value = serde_json::from_str(&resp.text()?).unwrap();

    // for repo in body.as_array().unwrap() {
        
    //     gh_db.push( Repo {
    //         name: trim_quotes(&repo["name"]),
    //         visibility: trim_quotes(&repo["visibility"]),
    //         owner: trim_quotes(&repo["owner"]["login"]),
    //     });
    // }




    // let gitea_username = "maxgallup";
    // let gitea_password = get_token("gitea-token.txt");
    // let gitea_client = reqwest::blocking::Client::builder()
    //     .default_headers(gitea_headers())
    //     .build()?;

    // // gitea_client.


    // for repo in gh_db {
    //     // migrate(repo)
    //     println!("{:?}", repo);
    // }

    Ok(())
}





