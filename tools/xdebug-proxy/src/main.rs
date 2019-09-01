//extern crate tokio;

use std::io::{self, Read, Write};
use std::net::{Shutdown, SocketAddr};
use std::sync::{Arc, Mutex};
use tokio::io::{copy, shutdown};
use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::*;

// https://github.com/tokio-rs/tokio/blob/2c4549a18ae1595fec6a1737b4257daedd37a8fa/tokio/examples/proxy.rs
fn main() {
    // Bind the server's socket.
    let addr = "0.0.0.0:9001".parse().unwrap();
    let listener = TcpListener::bind(&addr).expect("unable to bind TCP listener");

    let host_addr = "10.0.2.2:9002".parse::<SocketAddr>().unwrap();

    // Pull out a stream of sockets for incoming connections
    let from_docker = listener
        .incoming()
        .map_err(|e| eprintln!("accept failed = {:?}", e))
        .for_each(move |sock| {
            println!("Got connection on 9001");

            // Connect to IDE on host
            let ide = TcpStream::connect(&host_addr);
            let amounts = ide.and_then(move |ide| {
                //let (php_reader, php_writer) = sock.split();
                //let (ide_reader, ide_writer) = ide.split();
                let php_reader = MyTcpStream(Arc::new(Mutex::new(sock)), "php");
                let php_writer = php_reader.clone();
                let ide_reader = MyTcpStream(Arc::new(Mutex::new(ide)), "ide");
                let ide_writer = ide_reader.clone();
                let php_to_ide = copy(php_reader, ide_writer)
                    .and_then(|(n, _, ide_writer)| {
                        println!("Done php > ide");
                        shutdown(ide_writer).map(move |_| n)});

                let ide_to_php = copy(ide_reader, php_writer)
                    .and_then(|(n, _, php_writer)| {
                        println!("Done ide > php");
                        shutdown(php_writer).map(move |_| n)});

                php_to_ide.join(ide_to_php)
            });
            let msg = amounts
                .map(move |(from_client, from_server)| {
                    println!(
                        "client wrote {} bytes and received {} bytes",
                        from_client, from_server
                    );
                })
                .map_err(|e| {
                    // Don't panic. Maybe the client just disconnected too soon.
                    println!("error: {}", e);
                });
            tokio::spawn(msg);
            Ok(())
        });

    // Start the Tokio runtime
    tokio::run(from_docker);
}

// This is a custom type used to have a custom implementation of the
// `AsyncWrite::shutdown` method which actually calls `TcpStream::shutdown` to
// notify the remote end that we're done writing.
#[derive(Clone)]
struct MyTcpStream(Arc<Mutex<TcpStream>>, &'static str);

impl Read for MyTcpStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        //println!("Read {}: {:?}, {}\n", self.1, unsafe{ String::from_utf8_unchecked(buf.to_vec()).trim_matches(char::from(0)) }, buf.len());
        self.0.lock().unwrap().read(buf)
    }
}

impl Write for MyTcpStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        println!("Write {}: {:?}, {}\n", self.1, unsafe{ String::from_utf8_unchecked(buf.to_vec()).trim_matches(char::from(0)) }, buf.len());
        self.0.lock().unwrap().write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl AsyncRead for MyTcpStream {}

impl AsyncWrite for MyTcpStream {
    fn shutdown(&mut self) -> Poll<(), io::Error> {
        self.0.lock().unwrap().shutdown(Shutdown::Write)?;
        Ok(().into())
    }
}
