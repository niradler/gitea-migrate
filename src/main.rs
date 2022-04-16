

use reqwest::{self, header::HeaderMap};
use reqwest::header::*;

use serde::Deserialize;

use serde_json;

use std::fs;
use std::io::prelude::*;
use std::io::BufReader;

use base64;


// #[derive(Deserialize, Debug)]
// struct Obj {
//     items: Vec<String>,
// }

// #[derive(Deserialize, Debug)]
// struct Repo {
//     html_url: String,
// }

fn get_token(path: &str) -> String {

    let inner = match fs::File::open(path) {
        Ok(file) => file,
        Err(e) => panic!("{:?}", e),
    };
    
    let mut reader = BufReader::new(inner);
    let mut buf: String = String::new();
    reader.read_line(&mut buf).expect("Unable to read line");

    buf
}

fn custom_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    let username = "maxgallup";
    let password = get_token("token.txt");
    
    let encoded_credentials = base64::encode(format!("{}:{}", username, password));
    let basic_auth = format!("Basic {}", encoded_credentials);
    
    headers.insert(AUTHORIZATION, HeaderValue::from_str(&basic_auth).unwrap());
    headers.insert(USER_AGENT, HeaderValue::from_static("api"));
    headers.insert(ACCEPT, HeaderValue::from_static("application/vnd.github.v3+json"));
    headers
}


fn main() -> Result<(), reqwest::Error> {


    let github_base = "https://api.github.com/";
    let user = "maxgallup";
    let all_repos = "https://api.github.com/user/repos?per_page=200";
    let _public_repo = format!("{}users/{}/repos", github_base, user);

    //"https://api.github.com/orgs/$GITHUB_ORGANISATION/repos?per_page=200&type=all"
  
    
    let gh_client = reqwest::blocking::Client::builder()
        .default_headers(custom_headers())
        .build()?;

    

    let resp = gh_client.get(all_repos).send()?;


    // println!("resp.status = {:?}", resp.status());
    // println!("resp.headers = {:?}", resp.headers());
    // println!("resp.text = {:?}", resp.text()?);
    let body : serde_json::Value = serde_json::from_str(&resp.text()?).unwrap();

    for repo in body.as_array().unwrap() {
        println!("{} {} {}", &repo["name"].to_string(), &repo["visibility"].to_string(), &repo["owner"]["login"].to_string());
        // println!("-----------------")
    }


    Ok(())
}





