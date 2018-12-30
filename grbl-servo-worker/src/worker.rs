use serial::prelude::*;
use std::io::{Write, Read};

use amqp::{Session, Options, Table, Basic, protocol, Channel};
use amqp::QueueBuilder;
use amqp::ConsumeBuilder;
use amqp::TableEntry::LongString;
use std::default::Default;

fn consumer_function(channel: &mut Channel, deliver: protocol::basic::Deliver, headers: protocol::basic::BasicProperties, body: Vec<u8>){
    println!("[function] Got a delivery:");
    println!("[function] Deliver info: {:?}", deliver);
    println!("[function] Content headers: {:?}", headers);
    println!("[function] Content body: {:?}", body);
    println!("[function] Content body(as string): {:?}", String::from_utf8(body.clone()));
    channel.basic_ack(deliver.delivery_tag, false).unwrap();
    
    let received = String::from_utf8(body).expect("Faile to convert message to string");

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
    
    for line in received.lines() {
            port.write(line.as_bytes())
                .expect("Failed to write to the serial port");
            let mut buf: Vec<u8> = (0..255).collect();
            port.read(&mut buf).expect("Couldnt read from serial port after write");
    }
}

pub fn work(){

    let mut props = Table::new();
    props.insert("gcode-consumer".to_owned(), LongString("consumer".to_owned()));
    let mut session = Session::new(Options{
        properties: props,
        vhost: "/".to_string(),
        .. Default::default()
    }).ok().expect("Can't create session");
    let mut channel = session.open_channel(1).ok().expect("Error openning channel 1");
    println!("Openned channel: {:?}", channel.id);

    let queue_name = "print";
    let queue_builder = QueueBuilder::named(queue_name).durable();
    let queue_declare = queue_builder.declare(&mut channel);

    println!("Queue declare: {:?}", queue_declare);
    channel.basic_prefetch(10).ok().expect("Failed to prefetch");
    println!("Declaring consumers...");
    
    let consume_builder = ConsumeBuilder::new(consumer_function, queue_name);
    let consumer_name = consume_builder.basic_consume(&mut channel);
    println!("Starting consumer {:?}", consumer_name);

    channel.start_consuming();

    channel.close(200, "Bye").unwrap();
    session.close(200, "Good Bye");

}