

use reqwest;

fn main() -> Result<(), reqwest::Error> {


    
    let body = reqwest::blocking::get("https://www.rust-lang.org")?
    .text()?;

    println!("body = {:?}", body);


    Ok(())
}
