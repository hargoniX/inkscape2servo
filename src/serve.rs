use rocket::config::{Config, Environment};
use rocket::{get, post, routes, State};
use rocket::http::Status;
use rocket_contrib::json::Json;
use serde_derive::*;
use serial::prelude::*;
use std::io::{Read, Write};
use std::sync::{Mutex, mpsc};
use std::thread;

#[get("/")]
fn index() -> String {
    String::from("This is the server of inkscape2servo up and running!")
}

#[derive(Serialize, Deserialize)]
struct Message {
    gcode: String,
}

#[post("/print", format = "json", data = "<msg>")]
fn print(msg: Json<Message>, tx_storage: State<ThreadSenderStorage>) -> Status{
    let tx = tx_storage.tx.lock();
    match tx {
        Err(poisoned) => Status::new(500, "Couldnt lock the Mutex as another thread already made it crash, you must restart the server"),
        Ok(tx) => {
            tx.send(msg.gcode.clone()).expect("Couldnt send message to worker");
            Status::new(200, "Your print task was forwarded to the serial worker")
        }
    }
}

fn serial_routine(rx: mpsc::Receiver<String>) {
    let mut port = serial::open("/dev/ttyAMA0")
        .expect("The Serial device /dev/ttyAMA0 doesnt exist on this machine");
    port.reconfigure(&|settings| {
        settings
            .set_baud_rate(serial::Baud115200)
            .expect("Failed to set baudrate on /dev/ttyAMA0");
        settings.set_char_size(serial::Bits8);
        Ok(())
    })
    .expect("Failed to configure /dev/ttyAMA0 properly");

    loop {
        let received = rx
            .recv()
            .expect("Faile to receive new content into serial routine");
        for line in received.lines() {
            port.write(line.as_bytes())
                .expect("Failed to write to the serial port");
            let mut buf: Vec<u8> = (0..255).collect();
            port.read(&mut buf).expect("Couldnt read from serial port after write");
        }
    }
}

struct ThreadSenderStorage {
    tx: Mutex<mpsc::Sender<String>>,
}

pub fn serve(ip: String, port: Option<u16>) {
    let (tx, rx) = mpsc::channel();
    let tx = Mutex::new(tx);

    thread::spawn(move || {
        serial_routine(rx);
    });

    let config = Config::build(Environment::Production)
        .address(ip)
        .port(match port {
            Some(x) => x,
            None => 8080,
        })
        .finalize()
        .expect("Failed to build config from provided IP and Port");

    let server = rocket::custom(config);
    server.mount("/", routes![print, index]).manage(ThreadSenderStorage{tx: tx}).launch();
}