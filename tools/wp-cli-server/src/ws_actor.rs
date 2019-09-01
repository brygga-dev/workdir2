use actix::prelude::*;
use actix_web_actors::ws;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

use crate::cmd_actor::{CmdActor, WsMsg};

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

pub struct MyWebSocket {
    hb: Instant,
    cmd_addr: Option<actix::Addr<CmdActor>>,
}
impl MyWebSocket {
    pub fn new() -> Self {
        Self {
            hb: Instant::now(),
            cmd_addr: None,
        }
    }

    fn hb(&self, ctx: &mut <Self as Actor>::Context) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // Check heartbeat
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                println!("Client heartbeat failed, disconnecting");
                ctx.stop();
                return;
            }
            ctx.ping("");
        });
    }
}

impl Actor for MyWebSocket {
    type Context = ws::WebsocketContext<Self>;
    fn started(&mut self, ctx: &mut Self::Context) {
        let ws_addr = ctx.address().clone();
        self.cmd_addr = Some(actix::SyncArbiter::start(1, move || {
            CmdActor::new(ws_addr.clone())
        }));
        self.hb(ctx);
    }
}

/// Message coming from cmd_actor,
/// containing output from commands
/// and signal that the command is
/// finished.
#[derive(Serialize, Deserialize)]
pub enum FromCmdMsg {
    Line(String),
    CmdDone,
    Info(String),
    AllDone,
}
impl Message for FromCmdMsg {
    type Result = ();
}
impl Handler<FromCmdMsg> for MyWebSocket {
    type Result = ();
    fn handle(&mut self, msg: FromCmdMsg, ctx: &mut Self::Context) {
        match serde_json::to_string(&msg) {
            Ok(json) => ctx.text(json),
            Err(e) => println!("Serialize error: {:?}", e),
        }
        /*
        match msg {
            FromCmdMsg::Line(line) => ctx.text(line),
            FromCmdMsg::CmdDone => ctx.text("Cmd done"),
            FromCmdMsg::Info(info) => ctx.text(info)
        }*/
    }
}

/// StreamHandler handles basic communication
/// with websocket
impl StreamHandler<ws::Message, ws::ProtocolError> for MyWebSocket {
    fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
        println!("WS: {:?}", msg);
        match msg {
            ws::Message::Ping(msg) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            ws::Message::Pong(_) => {
                self.hb = Instant::now();
            }
            ws::Message::Text(text) => {
                let ws_msg = match serde_json::from_str::<WsMsg>(&text) {
                    Ok(ws_msg) => ws_msg,
                    Err(e) => {
                        println!("Ws msg deserialize error: {:?}", e);
                        return;
                    }
                };
                match &self.cmd_addr {
                    Some(cmd_addr) => {
                        println!("Sending ws_msg");
                        cmd_addr.do_send(ws_msg);
                    }
                    None => (),
                }
            }
            ws::Message::Binary(bin) => ctx.binary(bin),
            ws::Message::Close(_) => {
                ctx.stop();
            }
            ws::Message::Nop => (),
        }
    }
}
