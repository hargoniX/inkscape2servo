use rocket::config::{Config, Environment};
use rocket::{get, post, routes};
use rocket_contrib::json::Json;
use serde_derive::*;
use amqp::{Session, Table, Basic, protocol};

// Just a generic "I am here" route
#[get("/")]
fn index() -> String {
    String::from("This is grbl-servo-server up and running!")
}

#[derive(Serialize, Deserialize)]
struct Message {
    gcode: String,
}

// Opens a session to RMQ and pushes the received gcode to the print queue
#[post("/print", format = "json", data = "<msg>")]
fn print(msg: Json<Message>){
    let mut session = Session::open_url("amqp://localhost//").unwrap();
    let mut channel = session.open_channel(1).unwrap();
    let queue_declare = channel.queue_declare("print", false, true, false, false, false, Table::new());
    channel.basic_publish("", "print", true, false, protocol::basic::BasicProperties{ content_type: Some("text".to_string()), ..Default::default()}, msg.gcode.as_bytes().to_vec()).expect("Failed to send message to rabbit mq");
}

// Configures and starts a rocket webserver
pub fn serve(ip: String, port: Option<u16>) {
    let config = Config::build(Environment::Production)
        .address(ip)
        .port(match port {
            Some(x) => x,
            None => 8080,
        })
        .finalize()
        .expect("Failed to build config from provided IP and Port");

    let server = rocket::custom(config);
    server.mount("/", routes![print, index]).launch();
}