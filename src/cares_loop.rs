/*
  Copyright 2016 Colm Hickey <colmohici@gmail.com>
  Licensed under the Apache License, Version 2.0 (the "License");
  you may not use this file except in compliance with the License.
  You may obtain a copy of the License at
      http://www.apache.org/licenses/LICENSE-2.0
  Unless required by applicable law or agreed to in writing, software
  distributed under the License is distributed on an "AS IS" BASIS,
  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
  See the License for the specific language governing permissions and
  limitations under the License.
*/

// basically entirely taken from https://github.com/dimbleby/rust-c-ares/blob/master/examples/unix/cares_epoll.rs

use std::sync::{Arc, Mutex};
use std::thread::spawn;
use std::collections::HashSet;

use nix::sys::epoll::{epoll_create, epoll_ctl, epoll_wait, EpollEvent, EpollEventKind, EpollOp, EPOLLIN, EPOLLOUT};
use c_ares;

pub fn start_cares() -> Arc<Mutex<c_ares::Channel>> {
    let mut options = c_ares::Options::new();
    options.set_flags(c_ares::flags::STAYOPEN).set_timeout(500).set_tries(3);

    let channel = Arc::new(Mutex::new(c_ares::Channel::new(options).unwrap()));
    let ret_channel = channel.clone();

    spawn(move || {
        let epoll = epoll_create().expect("Failed to create epoll");
        let mut tracked_fds = HashSet::<c_ares::Socket>::new();
		loop {
			// Ask c-ares what file descriptors we should be listening on, and map
			// those requests onto the epoll file descriptor.
			for (fd, readable, writable) in &channel.lock().unwrap().get_sock() {
				let mut interest = EpollEventKind::empty();
				if readable { interest = interest | EPOLLIN; }
				if writable { interest = interest | EPOLLOUT; }
				let event = EpollEvent {
					events: interest,
					data: fd as u64,
				};
				let op = if tracked_fds.insert(fd) {
					EpollOp::EpollCtlAdd
				} else {
					EpollOp::EpollCtlMod
				};
				epoll_ctl(epoll, op, fd, &event).expect("epoll_ctl failed");
			}

			// Wait for something to happen.
			let empty_event = EpollEvent {
				events: EpollEventKind::empty(),
				data: 0,
			};
			let mut events = [empty_event; 2];
			let results = epoll_wait(epoll, &mut events, 500)
				.expect("epoll_wait failed");

			// Process whatever happened.
			match results {
				0 => {
					// No events - must be a timeout.  Tell c-ares about it.
					channel.lock().unwrap().process_fd(
						c_ares::SOCKET_BAD,
						c_ares::SOCKET_BAD);
				},
				n => {
					// Sockets became readable or writable.  Tell c-ares.
					for event in &events[0..n] {
						let active_fd = event.data as c_ares::Socket;
						let rfd = if (event.events & EPOLLIN).is_empty() {
							c_ares::SOCKET_BAD
						} else {
							active_fd
						};
						let wfd = if (event.events & EPOLLOUT).is_empty() {
							c_ares::SOCKET_BAD
						} else {
							active_fd
						};
						channel.lock().unwrap().process_fd(rfd, wfd);
					}
				}
			}
		} 
    });

    ret_channel
}
