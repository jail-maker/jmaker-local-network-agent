extern crate ipnetwork;
#[macro_use]
extern crate serde_json;
extern crate actix_web;
extern crate clap;

use ipnetwork::Ipv4Network;
use std::process::Command;
use serde_json::{Value, Error};

use actix_web::{http, server, App as WebApp, Path, HttpRequest, HttpResponse};
use clap::{Arg, App as ClapApp, SubCommand};

fn type_arg(arg: i32) {}

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

    use std::sync::Arc;

    let iface = Arc::new(argv.value_of("iface").unwrap().to_string());
    // let network = argv.value_of("network").unwrap();
    let bind = argv.value_of("bind").unwrap().to_string();

    // type_arg(iface);
    let iface2 = iface.clone();

    server::new(|| {

        WebApp::new()
            .route("/", http::Method::GET, |_request: HttpRequest| {

                println!("{:?}", iface);


                let ip = get_free_ip("lo0").unwrap();
                let body = json!({
                    "ip": ip,
                    "interface": "lo0"
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

            println!("{:?}", object["address"]);
            object["address"].as_str().unwrap().to_string()

        })
        .collect::<Vec<String>>();

    ips

}

fn get_free_ip(iface: &str) -> Option<String> {

    let ips = get_iface_ips(iface);
    let ip = "127.0.0.0".parse().unwrap();
    let network = Ipv4Network::new(ip, 8).unwrap();
    let broadcast = network.broadcast();
    let network_addr = network.network();
    let free_ip = network.iter().find(|ip| {

        if ip == &broadcast || ip == &network_addr {
            return false;
        }

        if ips.contains(&ip.to_string()) { false }
        else { true }

    });

    Some(free_ip.unwrap().to_string())

}
