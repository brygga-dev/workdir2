use actix::prelude::*;
use serde::Deserialize;
use std::net::TcpStream;
use std::net::SocketAddr;

use crate::ws_actor::{FromCmdMsg, MyWebSocket};

// It (should?) be possible to get the
// address of the calling actor instead
pub struct CmdActor {
    ws_addr: actix::Addr<MyWebSocket>,
    wait_for: Option<SocketAddr>,
    has_waited_for: bool,
}
impl CmdActor {
    pub fn new(ws_addr: actix::Addr<MyWebSocket>) -> Self {
        let wait_for = match std::env::var("WAIT_FOR") {
            Ok(wait_for) => {
                use std::net::ToSocketAddrs;
                match wait_for.to_socket_addrs() {
                    Ok(mut addr) => {
                        let addr = addr.next();
                        println!("Received wait for address: {:?}", addr);
                        addr
                    },
                    Err(e) => panic!(format!("Could not parse WAIT_FOR to socket address: {}, {:?}", e, wait_for))
                }
            },
            Err(_e) => None
        };
        CmdActor {
            ws_addr,
            wait_for,
            has_waited_for: true,
        }
    }
}
impl Actor for CmdActor {
    type Context = SyncContext<Self>;
    fn started(&mut self, _ctx: &mut Self::Context) {
        println!("Started CmdActor");
    }
}

#[derive(Deserialize)]
pub enum WsMsg {
    CmdMsg(CmdMsg),
    DoneMsg,
}
/// Message to run a command
#[derive(Deserialize, Debug)]
pub struct CmdMsg {
    pub cmd: String,
    pub args: Vec<String>,
}
impl Message for WsMsg {
    type Result = Option<usize>;
}
use std::io::prelude::*;
impl Handler<WsMsg> for CmdActor {
    type Result = Option<usize>;

    fn handle(&mut self, msg: WsMsg, _ctx: &mut SyncContext<Self>) -> Self::Result {
        let cmd_msg = match msg {
            WsMsg::CmdMsg(cmd_msg) => cmd_msg,
            WsMsg::DoneMsg => {
                // Signals all commands has gone in,
                // so has now been processed
                self.ws_addr.do_send(FromCmdMsg::AllDone);
                return None;
            }
        };
        // In this case, we need to wait for a connection
        // to database.
        if let (Some(wait_for), false) = (&self.wait_for, self.has_waited_for) {
            let mut attempts = 0;
            let max_attempts = 15;
            loop {
                match TcpStream::connect(wait_for) {
                    Ok(_) => break,
                    Err(e) => {
                        // todo: Investigate ctx.run_later, take attempt arg
                        // BUT, I guess it would interfere with order
                        attempts = attempts + 1;
                        if attempts >= max_attempts {
                            self.ws_addr.do_send(FromCmdMsg::Info(format!(
                                "Aborting after max attempts: {}",
                                max_attempts
                            )));
                            break;
                        }
                        self.ws_addr.do_send(FromCmdMsg::Info(format!(
                            "Couldn't connect, retrying, error: {:?}",
                            e
                        )));
                        std::thread::sleep(std::time::Duration::from_millis(1500));
                        // Could do_send to self here
                    }
                }
            }
            self.has_waited_for = true;
        }
        println!("Spawning cmd, {:?}", &cmd_msg);
        // ws/actors might not be the best abstraction for this,
        // for example interrupts would be nice, and clunky with this
        let mut process = std::process::Command::new(cmd_msg.cmd)
            .args(cmd_msg.args)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        let stdout = process.stdout.take().unwrap();
        let reader = std::io::BufReader::new(stdout);
        for line in reader.lines() {
            self.ws_addr.do_send(FromCmdMsg::Line(line.unwrap()));
        }
        println!("Cmd done");
        self.ws_addr.do_send(FromCmdMsg::CmdDone);
        None
    }
}
