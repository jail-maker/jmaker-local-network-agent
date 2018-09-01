extern crate ipnetwork;
#[macro_use]
extern crate serde_json;
extern crate actix_web;

use ipnetwork::Ipv4Network;
use std::process::Command;
use serde_json::{Value, Error};

use actix_web::{http, server, App, Path, HttpRequest, HttpResponse};

fn main() {

    server::new(|| {

        App::new()
            .route("/", http::Method::GET, |_request: HttpRequest| {

                let ip = get_free_ip("lo0").unwrap();
                let body = json!({
                    "ip": ip,
                    "interface": "lo0"
                });

                HttpResponse::Ok()
                    .header("content-type", "application/json")
                    .body(body.to_string())

            })

    }).bind("127.0.0.1:3000").unwrap().run();

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
    let ip = "127.0.0.1".parse().unwrap();
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
