#[macro_use]
extern crate tantivy;

mod chunked_file;
mod proxy;
mod search;
mod util;

use proxy::*;

use actix_cors::Cors;
use actix_web::{http::uri::Uri, web, App, HttpRequest, HttpServer};
use futures::future::Either;
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use util::uri_to_string;

// Tentative setup
#[derive(Clone)]
pub struct StaticResolved {
    file: PathBuf,
    if_webp: Option<PathBuf>,
}

//#[derive(Clone)]
pub struct AppData {
    cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
    cache_dep_id: Arc<RwLock<HashMap<u32, HashSet<String>>>>,
    cache_dep_pod: Arc<RwLock<HashMap<String, HashSet<String>>>>,
    cache_timeouts: Arc<RwLock<BTreeMap<i64, HashSet<String>>>>,
    path_configs: Arc<RwLock<HashMap<PathConfigKey, PathConfig>>>,
    static_resolved: Arc<RwLock<HashMap<String, Option<StaticResolved>>>>,
    index_data: search::AllIndexData,
    cache_dir: String,
    img_exp_minute_interval: u32,
    page_exp_minute_interval: u32,
    dev_mode: bool,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct PathConfigKey(String);

#[derive(Clone)]
struct PathConfig {
    request_base: String,
    key: PathConfigKey,
    url_path_prefix: String,
    replacements: Vec<(String, String)>,
    forward: HashSet<String>,
    static_paths: Option<Vec<(String, String)>>,
}

use clap::{self, Arg};

// todo: consider backtrace crate
fn main() {
    let matches = clap::App::new("Proxy server")
        .version("0.1")
        .author("Gudmund")
        .about("Proxies wordpress and other things to some other url with caching")
        .arg(
            Arg::with_name("wp")
                .short("w")
                .long("wordpress")
                .value_name("WORDPRESS")
                .env("WORDPRESS")
                .help("Wordpress url")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("bind")
                .short("b")
                .long("bind")
                .value_name("BIND")
                .env("BIND")
                .help("Bind server executable to this network address")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("external")
                .short("e")
                .long("external")
                .value_name("EXTERNAL")
                .env("EXTERNAL")
                .help("External address used to connect to site")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("uploads-path")
                .short("u")
                .long("uploads-path")
                .value_name("UPLOADS_PATH")
                .env("UPLOADS_PATH")
                .help("Path to wordpress' upload directory")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("cache-dir")
                .short("c")
                .long("cache-dir")
                .value_name("CACHE_DIR")
                .env("CACHE_DIR")
                .help("Where to place cache files")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("ssl")
                .short("s")
                .long("ssl")
                .value_name("SSL")
                .env("SSL")
                .help("Whether to serve over https")
                .required(false)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("dev-mode")
                .short("d")
                .long("dev-mode")
                .value_name("DEV_MODE")
                .env("DEV_MODE")
                .help("Dev mode with no proxy caching")
                .required(false)
                .takes_value(true),
        )
        .get_matches();

    // Wp url
    let wp_uri: Uri = match matches.value_of("wp") {
        Some(wp) => match wp.parse::<Uri>() {
            Ok(uri) => uri,
            Err(err) => {
                println!("Could not parse wp uri: {}", err);
                std::process::exit(2);
            }
        },
        None => {
            println!("Url of wordpress is required");
            std::process::exit(2);
        }
    };
    let (proxied, proxied_no_scheme) = match (
        uri_to_string(wp_uri.clone(), true),
        uri_to_string(wp_uri, false),
    ) {
        (Ok(uri), Ok(no_scheme)) => (uri, no_scheme),
        _ => {
            println!("Could not parse wp uri, scheme://host[:port]");
            std::process::exit(2);
        }
    };
    let bind_uri: Uri = match matches.value_of("bind") {
        Some(bind) => match bind.parse::<Uri>() {
            Ok(uri) => uri,
            Err(err) => {
                println!("Could not parse bind uri: {}", err);
                std::process::exit(2);
            }
        },
        None => {
            println!("Bind address is required");
            std::process::exit(2);
        }
    };
    // External uri

    // Fallback to bind (does this make (enough) sense?)
    // Hm, a simpler solution really is just to strip
    // the url generally..
    let external: Uri = match matches.value_of("external") {
        Some(external) => match external.parse::<Uri>() {
            Ok(uri) => uri,
            Err(err) => {
                println!("Could not parse external uri: {}", err);
                std::process::exit(2);
            }
        },
        None => bind_uri.clone(),
    };
    let external_uri = match uri_to_string(external, true) {
        Ok(uri) => uri,
        Err(_) => {
            println!("Could not parse external uri, scheme://host[:port]");
            std::process::exit(2);
        }
    };
    // Cache dir
    let cache_dir: String = match matches.value_of("cache-dir") {
        Some(cache_dir) => cache_dir.to_owned(),
        None => {
            println!("Cache dir is required");
            std::process::exit(2);
        }
    };
    // Uploads path
    let uploads_path: String = match matches.value_of("uploads-path") {
        Some(uploads_path) => uploads_path.to_owned(),
        None => {
            println!("Wordpress upload path is required");
            std::process::exit(2);
        }
    };
    // Ssl
    let ssl: bool = match matches.value_of("ssl") {
        Some(ssl) => match ssl {
            "0" | "false" | "no" => false,
            _ => true,
        },
        None => false,
    };
    // Dev mode
    let dev_mode: bool = match matches.value_of("dev-mode") {
        Some(dev_mode) => match dev_mode {
            "0" | "false" | "no" => false,
            _ => true,
        },
        None => false,
    };
    let (bind, bind_no_scheme) = match (
        bind_uri.scheme_part().map(|s| s.as_str()),
        bind_uri.host(),
        bind_uri.port_u16(),
    ) {
        (Some(scheme), Some(host), None) => {
            (String::from(scheme) + "://" + host, String::from(host))
        }
        (Some(scheme), Some(host), Some(port)) => (
            String::from(scheme) + "://" + host + ":" + &port.to_string(),
            String::from(host) + ":" + &port.to_string(),
        ),
        _ => {
            println!("Could not parse bind uri, scheme://host[:port]");
            std::process::exit(2);
        }
    };

    let cache: Arc<RwLock<HashMap<String, CacheEntry>>> = Arc::new(RwLock::new(HashMap::new()));
    let cache_dep_id: Arc<RwLock<HashMap<u32, HashSet<String>>>> =
        Arc::new(RwLock::new(HashMap::new()));
    let cache_dep_pod: Arc<RwLock<HashMap<String, HashSet<String>>>> =
        Arc::new(RwLock::new(HashMap::new()));
    let cache_timeouts: Arc<RwLock<BTreeMap<i64, HashSet<String>>>> =
        Arc::new(RwLock::new(BTreeMap::new()));
    // Path configs, keyed by request_base
    let path_configs: Arc<RwLock<HashMap<PathConfigKey, PathConfig>>> =
        Arc::new(RwLock::new(HashMap::new()));
    let static_resolved: Arc<RwLock<HashMap<String, Option<StaticResolved>>>> =
        Arc::new(RwLock::new(HashMap::new()));
    // Path config keys
    // todo: Could DRY this up a little, there is also
    // stuff in the services
    let wp_path_key = PathConfigKey("".into());
    let google_font_path_key = PathConfigKey("googlefont".into());
    let google_static_path_key = PathConfigKey("googlestatic".into());
    let wp_admin_key = PathConfigKey("wp-admin".into());
    let wp_json_key = PathConfigKey("wp-json".into());
    {
        let mut write_path_configs = path_configs.write().unwrap();
        // Root (wp client)
        use std::iter::FromIterator;
        write_path_configs.insert(
            wp_path_key.clone(),
            PathConfig {
                request_base: proxied.clone(),
                url_path_prefix: String::from(""),
                key: wp_path_key,
                replacements: vec![
                    (proxied.clone(), external_uri.clone()),
                    (
                        String::from("https://fonts.googleapis.com"),
                        external_uri.clone() + "/googlefont",
                    ),
                ],
                forward: HashSet::from_iter(
                    vec!["wp-login.php".into(), "wp-admin".into()].into_iter(),
                ),
                static_paths: Some(vec![("wp-content/uploads".into(), uploads_path)]),
                //static_paths: None
            },
        );
        // Google fonts
        write_path_configs.insert(
            google_font_path_key.clone(),
            PathConfig {
                request_base: String::from("https://fonts.googleapis.com"),
                url_path_prefix: String::from("googlefont"),
                key: google_font_path_key,
                replacements: vec![
                    (
                        String::from("https://fonts.googleapis.com"),
                        external_uri.clone() + "/googlefont",
                    ),
                    (
                        String::from("https://fonts.gstatic.com"),
                        external_uri.clone() + "/googlestatic",
                    ),
                ],
                forward: HashSet::new(),
                static_paths: None,
            },
        );
        // Google static
        write_path_configs.insert(
            google_static_path_key.clone(),
            PathConfig {
                request_base: String::from("https://fonts.gstatic.com"),
                url_path_prefix: String::from("googlestatic"),
                key: google_static_path_key,
                replacements: vec![],
                forward: HashSet::new(),
                static_paths: None,
            },
        );
        // Wp-admin
        write_path_configs.insert(
            wp_admin_key.clone(),
            PathConfig {
                request_base: proxied.clone() + "/wp-admin",
                url_path_prefix: String::from("wp-admin"),
                key: wp_admin_key,
                replacements: vec![(proxied.clone(), external_uri.clone())],
                forward: HashSet::new(),
                static_paths: None,
            },
        );
        // Wp-json
        write_path_configs.insert(
            wp_json_key.clone(),
            PathConfig {
                request_base: proxied.clone() + "/wp-json",
                url_path_prefix: String::from("wp-json"),
                key: wp_json_key,
                replacements: vec![(proxied.clone(), external_uri.clone())],
                forward: HashSet::new(),
                static_paths: None,
            },
        );
    }
    let sys = actix_rt::System::new("proxy");
    let index_data = search::initial_index_data();

    let ssl_builder = if ssl {
        // Ssl builder
        let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
        builder
            .set_private_key_file("key.pem", SslFiletype::PEM)
            .unwrap();
        builder.set_certificate_chain_file("cert.pem").unwrap();
        Some(builder)
    } else {
        None
    };

    // Setting up services
    let server = HttpServer::new(move || {
        let replace_host = proxied_no_scheme.clone();
        App::new()
            .data(AppData {
                cache: cache.clone(),
                cache_dep_id: cache_dep_id.clone(),
                cache_dep_pod: cache_dep_pod.clone(),
                cache_timeouts: cache_timeouts.clone(),
                path_configs: path_configs.clone(),
                static_resolved: static_resolved.clone(),
                index_data: index_data.clone(),
                cache_dir: cache_dir.clone(),
                img_exp_minute_interval: 30,
                page_exp_minute_interval: 3,
                dev_mode,
            })
            .service(web::resource("/--id-changed/{id}").to_async(id_changed)) // todo: post
            .service(web::resource("/--pod-added/{pod}").to_async(pod_added))
            .service(web::resource("/--clear-cache").to(clear_cache))
            .service(web::resource("/--index-menus").to_async(search::index_menus))
            .service(web::resource("/search").to(search::search_handler))
            .wrap(Cors::new())
            .service(web::resource("/googlefont/{url_path:.*}").to_async(
                |req: HttpRequest, data: web::Data<AppData>, payload: web::Payload| {
                    do_request_std(data, req, payload, PathConfigKey("googlefont".into()))
                },
            ))
            .service(web::resource("/googlestatic/{url_path:.*}").to_async(
                |req: HttpRequest, data: web::Data<AppData>, payload: web::Payload| {
                    do_request_std(data, req, payload, PathConfigKey("googlestatic".into()))
                },
            ))
            .service(web::resource("/wp-admin/{url_path:.*}").to_async(
                |req: HttpRequest, data: web::Data<AppData>, payload: web::Payload| {
                    do_request_forward(data, req, payload, PathConfigKey("wp-admin".into()), None)
                },
            ))
            .service(web::resource("/wp-json/{url_path:.*}").to_async(
                |req: HttpRequest, data: web::Data<AppData>, payload: web::Payload| {
                    do_request_forward(data, req, payload, PathConfigKey("wp-json".into()), None)
                },
            ))
            .service(web::resource("/{url_path:.*}").to_async(
                move |req: HttpRequest, data: web::Data<AppData>, payload: web::Payload| {
                    // Quick exception here for a page in customizer
                    if let Some(query) = req.uri().query() {
                        println!("Query {}", &query);
                        if query.starts_with("customize_changeset_uuid") {
                            clear_cache_data(&data);
                            return Either::A(do_request_forward(
                                data,
                                req,
                                payload,
                                PathConfigKey("".into()),
                                Some(replace_host.clone()),
                            ));
                        }
                    }
                    Either::B(do_request_std(data, req, payload, PathConfigKey("".into())))
                },
            ))
    });
    let server = if let Some(ssl_builder) = ssl_builder {
        server.bind_ssl(bind_no_scheme, ssl_builder)
    } else {
        server.bind(bind_no_scheme)
    };
    match server {
        Ok(server) => {
            server.start();
            println!("Listening on {}", bind);
            match sys.run() {
                Ok(()) => (),
                Err(err) => {
                    println!("Failed running system: {:?}", err);
                    std::process::exit(1);
                }
            }
        }
        Err(err) => {
            println!("Failed binding server: {:?}", err);
            std::process::exit(1);
        }
    }
}
