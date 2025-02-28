#![allow(non_snake_case)]
use socket2::{Domain, Protocol, Socket, Type};
use std::env;
use std::io::{self, Write};
use std::fs::OpenOptions;
use std::net::SocketAddr;
use std::mem::MaybeUninit;
use std::time::SystemTime;
use std::fs::create_dir_all;

enum IPVersion {
    IPV4,
    IPV6,
}


fn main() -> io::Result<()> {
    // IO
    let usage_message = "Usage: server <ip-version> <ip-protocol>";
    let ip_version = env::args().nth(1).expect(usage_message).parse::<u8>().map(
        |ip_version| match ip_version {
            4 => IPVersion::IPV4,
            6 => IPVersion::IPV6,
            _ => panic!("AFHIOAUWEBFGIUAEHGFAIEBFO I'm a panic message"),
        },
    ).expect("use a valid ip-version—4 or 6. ipv4 deprecation comin real soon.. :(");
    let ip_protocol = env::args().nth(2).expect(usage_message).parse::<i32>().expect("use a valid proto dummy");

    let domain = match ip_version {
        IPVersion::IPV4 => Domain::IPV4,
        IPVersion::IPV6 => Domain::IPV6,
    };

    // Init
    let socket = Socket::new(domain, Type::RAW, Some(Protocol::from(ip_protocol)))?;

    // Bind to the first interface on the system.
    let bind_addr = match domain {
        Domain::IPV4 => SocketAddr::from(([0, 0, 0, 0], 0)),
        Domain::IPV6 => SocketAddr::from(([0, 0, 0, 0, 0, 0, 0, 0], 0)),
        _ => panic!("aaaa mysterious error so you won't know what's wrong"),
    };

    socket.bind(&bind_addr.into())?;

    let mut buffer: [MaybeUninit<u8>; 65535] = unsafe { MaybeUninit::uninit().assume_init() };

    println!("Server is listening on {:?}, protocol: {}", socket.local_addr()?, ip_protocol);

    // This will be relevant later, trust me
    let dir_path = "/tmp/hdp";
    create_dir_all(dir_path)?;


    // // Loop time
    println!("| Protocol Number | Time (μs) (Server) | Source IP (Server) | Byte Sum (Server) |");
    println!("| --- | --- | --- |");

    loop {
        // Receive raw IP packets into `buffer`.
        let (size, _) = socket.recv_from(&mut buffer)?;

        // What's the time?
        let time_right_after_receiving_the_packet = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_micros();

        // Dear sir, this might appear unsafe, but we are only reading the buffer up to `size`
        // So we're not reading uninitialized memory.
        // Let's not bother the uninitialized memories, shall we?
        let buffer = unsafe { &*(buffer.as_ptr() as *const [u8; 65535]) };
        let received_data = &buffer[..size];

        // Let's write it in /tmp/hdp/ folder
        // (Yes, I'm a boogyman who uses unwrap like this)
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open(format!("/tmp/hdp/hdp_{time_right_after_receiving_the_packet}.bin"))?;
        file.write_all(received_data)?;
        // // PAAAAARSE
        let (ip__protocol, formatted_src_ip) = match ip_version {
            IPVersion::IPV4 => {
                let ip_header = &received_data[..20];
                let ip__protocol = ip_header[9];
                let ip__src_ip = &ip_header[12..16];
                let formatted_src_ip = format!("{}.{}.{}.{}", ip__src_ip[0], ip__src_ip[1], ip__src_ip[2], ip__src_ip[3]);
                (ip__protocol, formatted_src_ip)
            }
            IPVersion::IPV6 => {
                // FOR SOME REASON, IPv6 header is stripped from the packet, but ipv4 isn't??
                // received_data literally just has the payload
                // I read online that I need to use packet sockets, but for my testing, it isn't needed. Just need to capture *something*
                let ip__protocol = 0 as u8;
                let ip__src_ip = "None".to_string();
                (ip__protocol, ip__src_ip.to_string())
            }
        };

        println!(
            "| {} | {} | {} | {} |",
            ip__protocol,
            time_right_after_receiving_the_packet,
            formatted_src_ip,
            size,
        );

    }
}
