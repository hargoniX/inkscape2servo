use serialport::prelude::*;
use std::time::Duration;
use std::io;
use std::io::Write;

use amqp::{Session, Options, Table, Basic, protocol, Channel};
use amqp::QueueBuilder;
use amqp::ConsumeBuilder;
use amqp::TableEntry::LongString;
use std::default::Default;

// Called upon any message received from RMQ
fn consumer_function(channel: &mut Channel, deliver: protocol::basic::Deliver, headers: protocol::basic::BasicProperties, body: Vec<u8>){
    println!("[function] Got a delivery:");
    println!("[function] Deliver info: {:?}", deliver);
    println!("[function] Content headers: {:?}", headers);
    println!("[function] Content body: {:?}", body);
    println!("[function] Content body(as string): {:?}", String::from_utf8(body.clone()));
    channel.basic_ack(deliver.delivery_tag, false).unwrap();
    
    let received = String::from_utf8(body).expect("Faile to convert message to string");
    
    // Create and configure the serial port interface
    let mut serial_port_settings: SerialPortSettings = Default::default();
    serial_port_settings.baud_rate = 115200;
    serial_port_settings.timeout = Duration::from_secs(5);

    // Attempt to open the serial connection to the CNC machine via /deV/ttyAMA0
    match serialport::open_with_settings("/dev/ttyAMA0", &serial_port_settings) {
        Ok(mut port) => {
            let mut serial_buf: Vec<u8> = vec![0; 1000];
            // Send all lines from the RMQ message through the serial port
            for line in received.lines() {
                // Attempt to write the current line
                match port.write(line.as_bytes()) {
                    Err(e) => {
                        if e.kind() == io::ErrorKind::TimedOut {
                            println!("CNC machine timed out while sending gcodes: {}", e);
                        }
                        else {
                            println!("Unknown error while sending gcodes: {}", e);
                        }
                    },
                    Ok(_) => (),
                }
                // Attempt to read the response from grbl
                match port.read(serial_buf.as_mut_slice()) {
                    Err(e) => {
                        if e.kind() == io::ErrorKind::TimedOut {
                            println!("CNC machine timed out while waiting for response: {}", e);
                        }
                        else {
                            println!("Unkown error while waiting for response: {}", e);
                        }
                        std::process::exit(1);
                    },
                    Ok(t) => io::stdout().write_all(&serial_buf[..t]).expect("Couldnt read a string from buf"),
                }
            }
        },
        Err(e) => {
            println!("Failed to open port, error: {}", e);
            std::process::exit(1)
        }
    }
}

// Opens a connection to RMQ, declares and starts the consumer for the print queue
pub fn work(){
    // Open connection to RMQ 
    let mut props = Table::new();
    props.insert("gcode-consumer".to_owned(), LongString("consumer".to_owned()));
    let mut session = Session::new(Options{
        properties: props,
        vhost: "/".to_string(),
        .. Default::default()
    }).ok().expect("Couldn't create RMQ session");
    let mut channel = session.open_channel(1).ok().expect("Error openning channel 1");
    println!("Openned channel: {:?}", channel.id);

    // Declare print queue if not existing
    let queue_name = "print";
    let queue_builder = QueueBuilder::named(queue_name).durable();
    let queue_declare = queue_builder.declare(&mut channel);
    println!("Queue declare: {:?}", queue_declare);
    channel.basic_prefetch(10).ok().expect("Failed to prefetch");
    println!("Declaring consumers...");
    
    // Declare the consumer for the print queue which calls
    // the consumer_function when the queue receivies a message
    let consume_builder = ConsumeBuilder::new(consumer_function, queue_name);
    let consumer_name = consume_builder.basic_consume(&mut channel);
    println!("Starting consumer {:?}", consumer_name);
    
    // Start the consumer
    channel.start_consuming();

    channel.close(200, "Bye").unwrap();
    session.close(200, "Good Bye");
}