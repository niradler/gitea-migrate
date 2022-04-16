

use reqwest::{self, header::HeaderMap};
use reqwest::header::*;

use serde::Deserialize;

use serde_json;


// #[derive(Deserialize, Debug)]
// struct Obj {
//     items: Vec<String>,
// }

// #[derive(Deserialize, Debug)]
// struct Repo {
//     html_url: String,
// }

fn custom_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    //Accept: application/vnd.github.v3+json
    
    headers.insert(USER_AGENT, HeaderValue::from_static("api"));
    headers.insert(ACCEPT, HeaderValue::from_static("application/vnd.github.v3+json"));
    headers
}


fn main() -> Result<(), reqwest::Error> {


    let github_base = "https://api.github.com/";
    let user = "maxgallup";

    let gh_client = reqwest::blocking::Client::builder()
        .default_headers(custom_headers())
        .build()?;

    

    let resp = gh_client.get(format!("{}users/{}/repos", github_base, user)).send()?;


    // println!("resp.status = {:?}", resp.status());
    // println!("resp.headers = {:?}", resp.headers());
    let body : serde_json::Value = serde_json::from_str(&resp.text()?).unwrap();

    for repo in body.as_array().unwrap() {
        println!("{}", &repo["name"].to_string());
        println!("-----------------")
    }

    

    // println!("resp.body = {:?}", resp.text());


    Ok(())
}
