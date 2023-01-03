use std::path::PathBuf;

use reqwest::header::*;
use reqwest::{self, header::HeaderMap};

use serde_json;
use serde_json::json;
use serde_json::Value;

use std::fs;
use std::io::prelude::*;
use std::io::BufReader;

use std::io;
use std::io::Write;

use base64;

use clap::Parser;

// use log::{info, warn}; // TODO Logging

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

    /// migrate fork repos
    #[clap(long)]
    fork: bool,

    /// migrate only public and private repos
    #[clap(long)]
    both: bool,

    /// migrate all repos including associated repos from other users/orgs
    #[clap(long)]
    all: bool,

    /// desitnation url of gitea server: "https://gitea.example.com"
    #[clap(short, long, value_name = "URL")]
    dest: String,

    /// print useful information to stdout TODO logging
    #[clap(short, long)]
    verbose: bool,

    /// (optional) uses a custom credentials file with: 
    /// "github_user:github_token" and "gitea_user:gitea_password"
    /// on separate lines (see README)
    #[clap(short, long, parse(from_os_str), value_name = "FILE")]
    creds: Option<PathBuf>,

    /// set if repository shouldn't be mirrored
    #[clap(short, long)]
    no_mirror: bool,
}

#[derive(Debug)]
struct Repo {
    name: String,
    visibility: String,
    owner: String,
}

#[allow(dead_code)]
#[derive(Debug)]
struct UserSettings {
    public: bool,
    private: bool,
    any_owner: bool,
    requires_token: bool,
    fork: bool,
}

#[allow(dead_code)]
#[derive(Debug)]
struct UserData {
    gh: String,
    gt: String,
    gh_pass: String,
    gt_pass: String,
}

fn trim_quotes(v: &serde_json::Value) -> String {
    v.to_string().trim_matches('\"').to_string()
}

fn github_headers(user: &UserData) -> HeaderMap {
    let mut headers = HeaderMap::new();

    let encoded_credentials = base64::encode(format!("{}:{}", &user.gh, &user.gh_pass));
    let basic_auth = format!("Basic {}", encoded_credentials);

    headers.insert(AUTHORIZATION, HeaderValue::from_str(&basic_auth).unwrap());
    headers.insert(USER_AGENT, HeaderValue::from_static("api"));
    headers.insert(ACCEPT, HeaderValue::from_static("application/vnd.github.v3+json"));
    headers
}

fn gitea_headers(user: &UserData) -> HeaderMap {
    let mut headers = HeaderMap::new();
    let encoded_credentials = base64::encode(format!("{}:{}", &user.gt, &user.gt_pass));
    let basic_auth = format!("Basic {}", encoded_credentials);

    headers.insert(AUTHORIZATION, HeaderValue::from_str(&basic_auth).unwrap());
    headers.insert(USER_AGENT, HeaderValue::from_static("api"));
    headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers
}

fn creds_from_file(path: PathBuf) -> Option<UserData> {
    let inner = match fs::File::open(&path) {
        Ok(file) => file,
        Err(e) => panic!("{:?}", e),
    };

    let mut reader = BufReader::new(inner);
    let mut github: String = String::new();
    let mut gitea: String = String::new();
    reader.read_line(&mut github).expect("Unable to read line");
    reader.read_line(&mut gitea).expect("Unable to read line");

    let github: Vec<&str> = github.trim().split(":").collect();
    if !(github.len() == 2) {
        // TODO Make custom error messages (macros?)
        eprintln!("ERROR: unable to read github username from {:?}", &path);
        return None;
    }

    let gitea: Vec<&str> = gitea.trim().split(":").collect();
    if !(gitea.len() == 2) {
        // TODO Make custom error messages (macros?)
        eprintln!("ERROR: unable to read gitea information from {:?}", &path);
        return None;
    }

    Some(UserData {
        gh: github[0].to_string(),
        gt: gitea[0].to_string(),
        gh_pass: github[1].to_string(),
        gt_pass: gitea[1].to_string(),
    })
}

fn ask_for_creds() -> Option<UserData> {
    let stdin = io::stdin();

    //TODO repeated code
    println!("No credentials file specified (--creds), please enter the following:");
    print!("GITHUB USERNAME: ");
    io::stdout().flush().unwrap();
    let mut gh_user = String::new();
    stdin.read_line(&mut gh_user).unwrap();

    print!("GITHUB TOKEN: ");
    io::stdout().flush().unwrap();
    let mut gh_pass = String::new();
    stdin.read_line(&mut gh_pass).unwrap();

    print!("GITEA USERNAME: ");
    io::stdout().flush().unwrap();
    let mut gt_user = String::new();
    stdin.read_line(&mut gt_user).unwrap();

    print!("GITEA PASSWORD: ");
    io::stdout().flush().unwrap();
    let mut gt_pass = String::new();
    stdin.read_line(&mut gt_pass).unwrap();

    Some(UserData {
        gh: gh_user.trim().to_string(),
        gt: gt_user.trim().to_string(),
        gh_pass: gh_pass.trim().to_string(),
        gt_pass: gt_pass.trim().to_string(),
    })
}

fn gitea_body(no_mirror: bool, r: &Repo, u: &UserData) -> Value {

    let mut mirror = true;
    if no_mirror { mirror = false; }

    let mut private = false;
    if r.visibility == "private" { private = true; }

    let url = format!("https://github.com/{}/{}", &r.owner, &r.name);
    
    json!({
        "auth_username": u.gh,
        "auth_password": u.gh_pass,
        "clone_addr": url,
        "repo_name": r.name,
        "repo_owner": u.gt,
        "auth_password": u.gh_pass,
        "mirror": mirror,
        "private": private,
        "service": "git",
        "wiki": true,
    })
}

fn main() -> Result<(), reqwest::Error> {
    let args = Args::parse();

    let options = UserSettings {
        public: !args.private || args.both || args.all,
        private: args.private || args.both || args.all,
        any_owner: args.all,
        requires_token: args.private || args.both || args.all,
        fork: args.fork,
    };

    let user: Option<UserData> = match args.creds {
        Some(path) => creds_from_file(path),
        None => ask_for_creds(),
    };

    let user = match user {
        Some(u) => u,
        None => return Ok(()),
    };

    let gh_base_url = "https://api.github.com/";
    let gt_url = format!("{}/api/v1/repos/migrate", args.dest);
    let gh_api_url = format!("{}user/repos?per_page=100", gh_base_url);
    
    let gh_client = reqwest::blocking::Client::builder()
        .default_headers(github_headers(&user))
        .build()?;

    let resp = gh_client.get(gh_api_url).send()?;

    let mut gh_db : Vec<Repo> = Vec::new();

    let body : serde_json::Value = serde_json::from_str(&resp.text()?).unwrap();

    let mut count = 1;
    while body.as_array().unwrap().len() != 0 {
        let gh_api_url = format!("{}user/repos?per_page=100&page={}", gh_base_url, count);
        count+=1;
        let gh_client = reqwest::blocking::Client::builder()
            .default_headers(github_headers(&user))
            .build()?;
    
        let resp = gh_client.get(gh_api_url).send()?;
    
        let body : serde_json::Value = serde_json::from_str(&resp.text()?).unwrap();

        if body.as_array().unwrap().len() == 0 {
            break;
        }
        for repo in body.as_array().unwrap() {
            let name = trim_quotes(&repo["name"]);

            let visibility = trim_quotes(&repo["visibility"]);
            let owner = trim_quotes(&repo["owner"]["login"]);
    
            let user_is_owner = owner == user.gh;
            let is_private = "private" == visibility;
            let mut not_fork = trim_quotes(&repo["fork"]) == "false";

            if options.fork {
                not_fork = true;
            }
            
            let repo = Repo {
                name,
                visibility,
                owner,
            };
            if not_fork {
                if options.any_owner {
                    gh_db.push(repo); // --all
                } else if options.public && options.private {
                    if user_is_owner { gh_db.push(repo); } // --both
                } else if options.private {
                    if user_is_owner && is_private { gh_db.push(repo); } // --private
                } else {
                    if user_is_owner && !is_private { gh_db.push(repo); } // --public
                }
            }

        }
    }


    let gitea_client = reqwest::blocking::Client::builder()
        .default_headers(gitea_headers(&user))
        .build()?;
    println!("found repos: {}", gh_db.len());
    for repo in gh_db {
        println!("Migrating: {}", repo.name);

        gitea_client.post(&gt_url)
            .json(&gitea_body(args.no_mirror, &repo, &user))
            .send()?;
        
    }


    Ok(())
}
