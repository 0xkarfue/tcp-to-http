use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{self, Sender};
use std::{io::Read, thread};

fn read_line_chunk(mut stream: TcpStream, tx: Sender<String>) {
    match stream.peer_addr() {
        Ok(addr) => println!("Stream connected with {}", addr),
        Err(e) => println!("Error connecting {}", e),
    }

    thread::spawn(move || {
        let mut buffer = [0u8; 8];
        let mut line = String::new();
        loop {
            match stream.read(&mut buffer) {
                Ok(0) => {
                    println!("Client disconnected!");
                    break;
                }
                Ok(n) => {
                    // println!("{}", n);
                    for &byte in &buffer[..n] {
                        if byte == b'\n' {
                            tx.send(line.clone()).unwrap();
                            line.clear();
                        } else {
                            line.push(byte as char);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error reading from stream: {}", e);
                    break;
                }
            }
        }
    });
}

fn main() {
    let (tx, rx) = mpsc::channel::<String>();

    thread::spawn(move || {
        for received in rx {
            println!("{}", received);
        }
    });

    // read_line_chunk(String::from("./messages.txt"), tx);
    let _listner = TcpListener::bind("127.0.0.1:42069").expect("Failed to bind to port 42069");

    for stream in _listner.incoming() {
        match stream {
            Ok(stream) => {
                println!("{:?}", stream);
                read_line_chunk(stream, tx.clone());
            },
            Err(e) => println!("{}", e),
        }
    }
}
