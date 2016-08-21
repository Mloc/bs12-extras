extern crate rustc_serialize;
extern crate zmq;
extern crate nix;
extern crate c_ares;

use std::sync::{Arc, Mutex};

mod config;
mod cares_loop;

fn main() {
    let config = config::load_config("dnsbl.conf");
    let ares_channel = cares_loop::start_cares();

    let mut ctx = zmq::Context::new();

    let mut external_sock = ctx.socket(zmq::ROUTER).unwrap();
    external_sock.bind(&config.zmq_endpoint).unwrap();

    let mut internal_sock_send = ctx.socket(zmq::PAIR).unwrap();
    let mut internal_sock_recv = ctx.socket(zmq::PAIR).unwrap();

    internal_sock_recv.bind("inproc://internal").unwrap();
    internal_sock_send.connect("inproc://internal").unwrap();
    let internal_sock_send = Arc::new(Mutex::new(internal_sock_send));

    loop {
        match zmq::poll(&mut [external_sock.as_poll_item(zmq::POLLIN), internal_sock_recv.as_poll_item(zmq::POLLIN)], -1) {
            Ok(_) => {
                match external_sock.recv_bytes(zmq::DONTWAIT) {
                    Ok(id) => {
                        match external_sock.recv_string(0) {
                            Ok(Ok(s)) => {
                                let mut rev = s.split(".").collect::<Vec<&str>>();
                                rev.reverse();
                                let rev = rev.join(".");

                                for blacklist in config.blacklists.iter() {
                                    let check_domain = format!("{}.{}", rev, blacklist.domain);

                                    let ip = s.clone();
                                    let ret_sock = internal_sock_send.clone();
                                    let blacklist = blacklist.clone();
                                    let id = id.clone();
                                    ares_channel.lock().unwrap().query_a(&check_domain, move |res| {
                                        match res {
                                            Ok(res) => {
                                                match res.iter().next() {
                                                    Some(r) => {
                                                        let reason = match blacklist.reasons.get(&format!("{}", r.ipv4().octets()[3])) {
                                                            Some(s) => s,
                                                            None => "Unknown",
                                                        };
                                                        
                                                        let msg = format!("{}:{}", ip, blacklist.message);
                                                        let msg = msg.replace("%r", reason);
                                                        let  msg = msg.replace("%i", &ip);

                                                        ret_sock.lock().unwrap().send(&id, zmq::SNDMORE).unwrap();
                                                        ret_sock.lock().unwrap().send_str(&msg, 0).unwrap();
                                                    },
                                                    None => {},
                                                }
                                            },
                                            Err(c_ares::AresError::ENOTFOUND) => {},
                                            e @ Err(_) => {
                                                e.unwrap();
                                            }
                                        }
                                    });
                                }
                            },
                            Ok(_) => {},
                            e @ Err(_) => {
                                e.unwrap().unwrap();
                            },
                        }
                    },
                    Err(zmq::Error::EAGAIN) => {},
                    e @ Err(_) => {
                        e.unwrap();
                    },
                };
                match internal_sock_recv.recv_bytes(zmq::DONTWAIT) {
                    Ok(id) => {
                        match internal_sock_recv.recv_bytes(0) {
                            Ok(m) => {
                                external_sock.send(&id, zmq::SNDMORE).unwrap();
                                external_sock.send(&m, 0).unwrap();
                            },
                            e @ Err(_) => {
                                e.unwrap();
                            },
                        };
                    },
                    Err(zmq::Error::EAGAIN) => {},
                    e @ Err(_) => {
                        e.unwrap();
                    },
                };
            }
            e @ Err(_) => {
                e.unwrap();
            },
        }
    }
}
