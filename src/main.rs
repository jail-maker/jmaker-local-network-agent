extern crate ipnetwork;
extern crate serde_json;
extern crate actix_web;

use ipnetwork::Ipv4Network;
use std::process::Command;
use serde_json::{Value, Error};

use actix_web::{http, server, App, Path, Responder};

fn main() {

    server::new(
        || App::new()
        .route("/{id}/{name}/index.html", http::Method::GET, index))
        .bind("127.0.0.1:3000").unwrap()
        .run();

}

fn _get_free_ip() {

    let output = Command::new("/usr/bin/netstat")
        .args(&["-I", "lo0", "-4", "-n", "--libxo", "json"])
        .output()
        .expect("netstat command failed to start");

    let string_data = String::from_utf8_lossy(&output.stdout);

    let json_data: Value = serde_json::from_str(&string_data).unwrap();

    let interface = json_data["statistics"]["interface"].as_array().unwrap();

    let ips = interface.iter()
        .map(|object| {

            println!("{:?}", object["address"]);
            object["address"].as_str().unwrap()

        })
        .collect::<Vec<&str>>();

    println!("{:?}", ips);

    let ip = "127.0.0.1".parse().unwrap();
    let network = Ipv4Network::new(ip, 8).unwrap();
    let broadcast = network.broadcast();
    let network_addr = network.network();
    let free_ip = network.iter().find(|ip| {

        if ip == &broadcast || ip == &network_addr {
            return false;
        }

        if ips.contains(&ip.to_string().as_str()) { false }
        else { true }

    });

    println!("free: {}", free_ip.unwrap());

}
