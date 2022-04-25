use std::path::PathBuf;

use reqwest::header::*;
use reqwest::{self, header::HeaderMap};

use serde::Deserialize;

use serde_json;

use std::collections::HashMap;
use std::fs;
use std::io::prelude::*;
use std::io::BufReader;

use std::io;
use std::io::Write;

use base64;

use clap::Parser;

/// Migrate git repositories to a Gitea instance
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// (default) migrate only public repos
    #[clap(long)]
    public: bool,

    /// migrate only private repos (requires Github Username password)
    #[clap(long)]
    private: bool,

    /// migrate only public and private repos (requires Github Username password)
    #[clap(long)]
    both: bool,

    /// migrate all repos, including associated repos (requires Github Username password)
    #[clap(long)]
    all: bool,

    /// name of the user/org to migrate repositories from
    #[clap(short, long, value_name = "NAME")]
    from: String,

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





fn trim_quotes(v: &serde_json::Value) -> String {
    v.to_string().trim_matches('\"').to_string()
}

// fn get_content(r: &Repo) -> HashMap<String, String> {

//     HashMap::new()
// }

// fn migrate(r: &Repo) {
//     gitea_client.post("ads").json(get_content(&repo)).send()?;
// }

#[derive(Debug)]
struct Repo {
    name: String,
    visibility: String,
    owner: String,
}

#[allow(dead_code)]
#[derive(Debug)]
struct RepoOptions {
    public: bool,
    private: bool,
    any_owner: bool,
    requires_token: bool,
}

#[allow(dead_code)]
#[derive(Debug)]
struct UserData {
    gh: String,
    gt: String,
    gh_pass: Option<String>,
    gt_pass: String,
}

fn github_headers(user: &UserData) -> HeaderMap {
    let mut headers = HeaderMap::new();
    let username = &user.gh;
    
    let password = match &user.gh_pass {
        Some(s) => s,
        None => "",
    };

    let encoded_credentials = base64::encode(format!("{}:{}", username, password));
    let basic_auth = format!("Basic {}", encoded_credentials);

    headers.insert(AUTHORIZATION, HeaderValue::from_str(&basic_auth).unwrap());
    headers.insert(USER_AGENT, HeaderValue::from_static("api"));
    headers.insert(ACCEPT, HeaderValue::from_static("application/vnd.github.v3+json"));
    headers
}

fn gitea_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("api"));
    headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers
}

fn creds_from_file(path: PathBuf, require_token: bool) -> Option<UserData> {
    let inner = match fs::File::open(&path) {
        Ok(file) => file,
        Err(e) => panic!("{:?}", e),
    };

    let mut reader = BufReader::new(inner);
    let mut github: String = String::new();
    let mut gitea: String = String::new();
    reader.read_line(&mut github).expect("Unable to read line");
    reader.read_line(&mut gitea).expect("Unable to read line");

    let mut token: Option<String> = None;
    let github: Vec<&str> = github.trim().split(":").collect();
    if !(github[0].len() > 0) {
        // TODO Make custom error messages (macros?)
        eprintln!("ERROR: unable to read github username from {:?}", &path);
        return None;
    }
    if require_token && github.len() == 2 {
        token = Some(github[1].to_string());
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
        gh_pass: token,
        gt_pass: gitea[1].to_string(),
    })
}

fn ask_for_creds(require_token: bool) -> Option<UserData> {
    let stdin = io::stdin();

    //TODO repeated code
    println!("No credentials file specified (--creds), please enter the following:");
    print!("GITHUB USERNAME: ");
    io::stdout().flush().unwrap();
    let mut gh_user = String::new();
    stdin.read_line(&mut gh_user).unwrap();

    let mut token: Option<String> = None;
    if require_token {
        print!("GITHUB TOKEN: ");
        io::stdout().flush().unwrap();
        let mut gh_pass = String::new();
        stdin.read_line(&mut gh_pass).unwrap();
        token = Some(gh_pass.trim().to_string());
    }

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
        gh_pass: token,
        gt_pass: gt_pass.trim().to_string(),
    })
}

fn main() -> Result<(), reqwest::Error> {
    let args = Args::parse();

    let gt_url = args.to_url;

    let options = RepoOptions {
        public: !args.private || args.both || args.all,
        private: args.private || args.both || args.all,
        any_owner: args.all,
        requires_token: args.private || args.both || args.all,
    };

    let user: Option<UserData> = match args.creds {
        Some(path) => creds_from_file(path, options.requires_token),
        None => ask_for_creds(options.requires_token),
    };

    let user = match user {
        Some(u) => u,
        None => return Ok(()),
    };

    let gh_base_url = "https://api.github.com/";
    let mut gh_api_url = String::new();

    if options.requires_token {
        gh_api_url = format!("{}user/repos?per_page=200", gh_base_url);
    } else {
        gh_api_url = format!("{}users/{}/repos", gh_base_url, user.gh);
    }

    let gh_client = reqwest::blocking::Client::builder()
        .default_headers(github_headers(&user))
        .build()?;

    let resp = gh_client.get(gh_api_url).send()?;
    

    // let gitea_base = "https://git.basingse.org/api/v1";
    // let all_repos = "https://api.github.com/user/repos?per_page=200";

    let mut gh_db : Vec<Repo> = Vec::new();


    // // println!("resp.status = {:?}", resp.status());
    // // println!("resp.headers = {:?}", resp.headers());
    let body : serde_json::Value = serde_json::from_str(&resp.text()?).unwrap();

    for repo in body.as_array().unwrap() {

        gh_db.push( Repo {
            name: trim_quotes(&repo["name"]),
            visibility: trim_quotes(&repo["visibility"]),
            owner: trim_quotes(&repo["owner"]["login"]),
        });
    }

    // let gitea_username = "maxgallup";
    // let gitea_password = get_token("gitea-token.txt");
    // let gitea_client = reqwest::blocking::Client::builder()
    //     .default_headers(gitea_headers())
    //     .build()?;

    // // gitea_client.

    for repo in gh_db {
        // migrate(repo)
        println!("{:?}", repo);
    }

    Ok(())
}
