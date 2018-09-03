#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate lazy_static;
extern crate ipnetwork;
extern crate actix_web;
extern crate clap;

use ipnetwork::Ipv4Network;
use std::process::Command;
use serde_json::{Value, Error};

use actix_web::{http, server, App as WebApp, HttpRequest, HttpResponse};
use clap::{Arg, App as ClapApp};

use std::collections::HashMap;
use std::time::{SystemTime, Duration};
use std::sync::Mutex;
use std::sync::Arc;

lazy_static! {

    static ref RESERVED: Mutex<HashMap<String, SystemTime>> = {
        Mutex::new(HashMap::new())
    };

}

#[derive(Debug)]
struct AppState {
    iface: String,
    network: String,
}

fn main() {

    let argv = ClapApp::new("jmaker local network agent")
        .arg(
            Arg::with_name("iface")
                .short("i")
                .long("iface")
                .help("interface name")
                .default_value("lo1")
                .takes_value(true)
                .required(true)
        )
        .arg(
            Arg::with_name("network")
                .short("n")
                .long("network")
                .default_value("127.0.0.0/8")
                .takes_value(true)
                .required(true)
        )
        .arg(
            Arg::with_name("bind")
                .short("b")
                .long("bind")
                .default_value("127.0.0.1:3000")
                .takes_value(true)
                .required(true)
        )
        .get_matches();

    let bind = argv.value_of("bind").unwrap().to_string();

    let state = Arc::new(AppState {
        iface: argv.value_of("iface").unwrap().to_string(),
        network: argv.value_of("network").unwrap().to_string(),
    });

    server::new(move || {

        WebApp::with_state(state.clone())
            .route("/api/v1/free-ip", http::Method::GET, |req: HttpRequest<Arc<AppState>>| {

                let state = req.state();

                let ip = get_free_ip(&state.iface, &state.network).unwrap();
                let body = json!({
                    "ip": ip,
                    "iface": state.iface
                });

                HttpResponse::Ok()
                    .header("content-type", "application/json")
                    .body(body.to_string())

            })

    }).bind(bind).unwrap().run();

}

fn get_iface_ips(iface: &str) -> Vec<String> {

    let output = Command::new("/usr/bin/netstat")
        .args(&["-I", iface, "-4", "-n", "--libxo", "json"])
        .output()
        .expect("netstat command failed to start");

    let string_data = String::from_utf8_lossy(&output.stdout);
    let json_data: Value = serde_json::from_str(&string_data).unwrap();
    let interface = json_data["statistics"]["interface"].as_array().unwrap();
    let ips = interface.iter()
        .map(|object| {

            object["address"].as_str().unwrap().to_string()

        })
        .collect::<Vec<String>>();

    ips

}

fn get_free_ip(iface: &str, network: &str) -> Option<String> {

    let ips = get_iface_ips(iface);
    let network: Ipv4Network = network.parse().unwrap();
    let broadcast = network.broadcast();
    let network_addr = network.network();

    let mut reserved = RESERVED.lock().unwrap();
    let duration = Duration::new(5, 0);
    let now = SystemTime::now();

    let free_ip = network.iter().find(|ip| {

        let ip_string = ip.to_string();
        let res = &reserved;

        match reserved.get(&ip_string) {

            Some(time) => {

                if time >= &now {

                    return false;

                } else {

                    // reserved.borrow_mut().remove(&ip_string);

                }

            },
            _ => {}

        };

        if ip == &broadcast || ip == &network_addr {
            return false;
        }

        if ips.contains(&ip_string) { 

            false 

        } else {

            reserved.insert(ip_string, now + duration);
            true 

        }

    });

    Some(free_ip.unwrap().to_string())

}
