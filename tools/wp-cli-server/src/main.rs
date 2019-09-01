use actix_web::{middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web_actors::ws;
use std::io;

mod cmd_actor;
mod ws_actor;

use ws_actor::MyWebSocket;

/// Server for wp-cli
/// Primarily meant for dev use
/// Operates over websockets,
/// partly to get into the code..
/// Feels a bit overkill, but more appropriate
/// than http. Some other network protocol/socket
/// might be better fit eventually.
/// Compared to ssh solution, this should
/// be a more limited surface, providing
/// opportunities to curate, as well
/// as intercept and add additional
/// logic. At the cost of flexibility
/// and implementation cost.
///
/// There is a "front-end" actor (ws_actor),
/// and a cmd_actor that runs the command.
/// The front-end is not blocked on the command.

fn ws_index(r: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    println!("r: {:?}", r);
    let res = ws::start(MyWebSocket::new(), &r, stream);
    println!("{:?}", res.as_ref().unwrap());
    res
}

fn index_file() -> impl Responder {
    let b = match std::fs::read_to_string("static/index.html") {
        Ok(s) => s,
        Err(_) => "".to_string(),
    };
    HttpResponse::build(actix_web::http::StatusCode::from_u16(200).unwrap())
        .header("content-type", "text/html")
        .body(b)
}

fn main() -> io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_server=info,actix_web=info");
    let sys = actix_rt::System::new("cli-server");
    env_logger::init();
    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default())
            .service(
                web::resource("/ws/").route(
                    web::get().to(|r: HttpRequest, stream: web::Payload| ws_index(r, stream)),
                ),
            )
            .service(web::resource("/").to(index_file))
    })
    .bind("0.0.0.0:5711")?
    .start();
    sys.run()
}
