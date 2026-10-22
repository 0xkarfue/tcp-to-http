use std::sync::mpsc::{self, Sender};
use std::{fs::File, io::Read, thread};


fn read_line_chunk(path: String, tx: Sender<String>) {
    thread::spawn(move || {
        let mut file = File::open(path).unwrap();
        let mut buffer = [0u8; 8];
        let mut line = String::new();
        loop {
            let bytes_read = file.read(&mut buffer).unwrap();
            if bytes_read == 0 {
                if !line.is_empty() {
                    tx.send(line.clone()).unwrap();
                }
                break;
            }
            for &byte in &buffer[..bytes_read] {
                if byte == b'\n' {
                    tx.send(line.clone()).unwrap();
                    line.clear();
                } else {
                    line.push(byte as char);
                }
            }
        }
    });
}

fn main() {
    let (tx, rx) = mpsc::channel::<String>();

    read_line_chunk(String::from("./messages.txt"), tx);

    for received in rx {
        println!("read: {}", received);
    }
}
