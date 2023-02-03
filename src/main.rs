use log::{debug, error, info};
use mouse_rs::{types::keys::Keys, Mouse};
use rouille::{
    input, router, try_or_400,
    websocket::{start, Message, Websocket},
    Response,
};
use simplelog::{
    ColorChoice, CombinedLogger, Config, LevelFilter, TermLogger, TerminalMode, WriteLogger,
};
use std::{
    collections::HashMap,
    fs::File,
    net::{Ipv4Addr, SocketAddr},
    thread::spawn, process::exit,
};
use touchpad_server::{get_client_page, parse_message, /* ICON_DATA */};
// use tray_icon::{TrayIconBuilder, icon::Icon};
// use image::{io::Reader as ImageReader, DynamicImage};
// use std::io::Cursor;

fn ws_handler(mut ws: Websocket, address: &SocketAddr) {
    info!("New connection from {}.", address);

    let mouse = Mouse::new();
    debug!("Created virtual mouse client for {}.", address);

    while let Some(message) = ws.next() {
        match message {
            Message::Text(text) => parse_message(text, &mouse),
            _ => {}
        }
    }
    info!("Connection to {} was closed.", address);
    mouse
        .release(&Keys::LEFT)
        .unwrap_or_else(|_| error!("Couldn't release LMB!"));
    mouse
        .release(&Keys::RIGHT)
        .unwrap_or_else(|_| error!("Couldn't release RMB!"));
}

fn main() {
    // Initialize the Logger
    CombinedLogger::init(vec![
        TermLogger::new(
            match std::env::args().find(|el| el == "-v") {
                Some(_) => LevelFilter::Info,
                None => LevelFilter::Off,
            },
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(
            LevelFilter::Debug,
            Config::default(),
            File::create("touchpad-server.log").unwrap(),
        ),
    ])
    .unwrap_or_else(|_| println!("Couldn't initialize the logger!"));

    debug!("Program started, writing the log.");

    // Get the config
    let config = {
        match config::Config::builder()
            .set_default("login", "admin")
            .unwrap()
            .set_default("password", "admin")
            .unwrap()
            .add_source(config::File::with_name("settings").required(false))
            .add_source(
                config::Environment::with_prefix("TOUCHPAD")
                    .try_parsing(true)
                    .separator("_"),
            )
            .build()
            {
                Ok(settings) => settings.try_deserialize::<HashMap<String, String>>().unwrap(),
                Err(_) => {
                    error!("Couldn't find the config file!");
                    println!("Couldn't find the config file! Exiting...");
                    exit(1);
                }
            }
    };

    // Initialize the tray icon
    // let icon = match ImageReader::new(Cursor::new(ICON_DATA)).with_guessed_format().unwrap().decode() {
    //     Ok(img) => if let DynamicImage::ImageRgba8(rgba_image) = img {
    //         rgba_image.into_vec()
    //     } else {
    //         error!("Couldn't load the icon!");
    //         print!("Couldn't load the icon!");
    //         exit(1);
    //     },
    //     Err(_) => {
    //         error!("Couldn't load the icon!");
    //         print!("Couldn't load the icon!");
    //         exit(1);
    //     }
    // };
    // let tray_icon = TrayIconBuilder::new()
    //     .with_tooltip("Touchpad Server")
    //     .with_icon(Icon::from_rgba(icon, 32, 32).unwrap())
    //     .build()
    //     .unwrap();

    // Start the server
    // On every request the server checks for authentication and, if none provided, responds with authenticaton page

    let ip = "0.0.0.0:6890";

    info!(
        "Server started at {}:6890.",
        local_ip_address::local_ip().unwrap_or(std::net::IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)))
    );

    rouille::start_server_with_pool(ip, Some(16), move |request| {
        router!(request,
            (GET) (/) => {
                debug!("Client {} requested the Client Page.", request.remote_addr());
                let auth = match input::basic_http_auth(request) {
                    Some(a) => a,
                    None => return Response::basic_http_auth_login_required("realm")
                };
                if Some(&auth.login) == config.get("login") && Some(&auth.password) == config.get("password") {
                    debug!("Valid password, continuing...");
                    Response::html(get_client_page())
                } else {
                    Response::basic_http_auth_login_required("realm")
                }
            },
            (GET) (/ws) => {
                let auth = match input::basic_http_auth(request) {
                    Some(a) => a,
                    None => return Response::basic_http_auth_login_required("realm")
                };
                if Some(&auth.login) == config.get("login") && Some(&auth.password) == config.get("password") {
                    debug!("Valid password, continuing...");
                    let address = request.remote_addr().to_owned();
                    debug!("Client {} requested the WebSocket.", address);
                    let (response, websocket) = try_or_400!(start::<String>(&request, None));

                    spawn(move || {
                        let ws = websocket.recv().unwrap();

                        ws_handler(ws, &address);
                    });

                    response
                } else {
                    Response::basic_http_auth_login_required("realm")
                }
            },
            _ => Response::empty_404()
        )
    });
}
