use quicli::prelude::*;
use reqwest;
use std::collections::HashMap;

pub fn upload(file: String, ip: String, port: Option<String>) {
    // Reads the input file and puts it into a JSON dictionary
    let mut json_post = HashMap::new();
    json_post.insert("gcode", read_file(file).expect("Couldnt read input file"));

    // Opens a sessions to the given http server and send the JSON dictionary to that server
    let client = reqwest::Client::new();
    let server_url = match port {
        Some(port) => format!("http://{}:{}/print", ip, port),
        None => format!("http://{}:{}/print", ip, "8080"),
    };
    println!("Uploading...");
    client
        .post(&server_url)
        .json(&json_post)
        .send()
        .expect("There was an error while sending the request to the server");
    println!("Done");
}