use quicli::prelude::*;
use std::collections::HashMap;
use reqwest;

pub fn upload(file: String, ip: String, port: Option<String>) {
    let mut json_post = HashMap::new();
    json_post.insert("gcode", read_file(file).expect("Couldnt read input file"));

    let client = reqwest::Client::new();
    let server_url = match port {
        Some(x) => format!("http://{}:{}/print", ip, x) ,
        None => format!("http://{}/print", ip),
    };
    println!("Uploading...");
    client.post(&server_url).json(&json_post).send().expect("There was an error during sending the request to the server");
    println!("Done");
}