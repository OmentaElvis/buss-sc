use std::{
    char::decode_utf16,
    io::{self, BufReader, Read},
    net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream},
};

use buss_protocol::{types::buss_flags::UTF16, BussAction, BussHeader, BussSettings, FromBytes};
use clap::{value_parser, Arg, Command};

fn main() {
    let port_arg = Arg::new("port")
        .short('p')
        .long("port")
        .value_parser(value_parser!(u16))
        .help("Port to listen at");

    let cmd = Command::new("buss-sanity-checker").arg(port_arg);

    let matches = cmd.get_matches();
    let port: &u16 = match matches.get_one("port") {
        Some(port) => port,
        None => &42069,
    };
    let address = SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), *port);
    let sock = match TcpListener::bind(address) {
        Ok(sock) => {
            println!("[+] Bussin at port {}", port);
            sock
        }
        Err(err) => {
            eprintln!("[-] Failed to start tcp socket at port {}: {}", port, err);
            return;
        }
    };
    loop {
        let conn = match sock.accept() {
            Ok(conn) => {
                println!("[+] Received a connection from: {}", conn.1);
                conn
            }
            Err(err) => {
                eprintln!("[-] Unexpected error encountered: {}", err);
                return;
            }
        };
        let stream = conn.0;
        let mut reader = BufReader::new(stream);
        if let Err(err) = process_request(&mut reader) {
            eprintln!("[-] Error occured while processing request: {}", err);
            continue;
        }
    }
}

fn action_to_string(value: BussAction) -> String {
    let string = match value {
        BussAction::Noop => "NOOP",
        BussAction::Read => "READ",
        BussAction::Write => "WRITE",
        BussAction::Modify => "MOFIFY",
        BussAction::Delete => "DELETE",
    };
    String::from(string)
}

fn u8_to_settings(value: u8) -> Option<BussSettings> {
    match value {
        0 => Some(BussSettings::BodyLength),
        1 => Some(BussSettings::Host),
        0xff => Some(BussSettings::Custom),
        _ => None,
    }
}
fn read_u32(stream: &mut BufReader<TcpStream>) -> io::Result<u32> {
    let mut buf: [u8; 4] = [0; 4];
    stream.read_exact(&mut buf)?;
    Ok(u32::from_be_bytes(buf))
}
fn _read_u16(stream: &mut BufReader<TcpStream>) -> io::Result<u16> {
    let mut buf: [u8; 2] = [0; 2];
    stream.read_exact(&mut buf)?;
    Ok(u16::from_be_bytes(buf))
}

fn has_flag(flags: u8, flag: u8) -> bool {
    (flags & flag) == flag
}

fn print_flags(flags: u8) {
    if has_flag(flags, buss_protocol::types::buss_flags::UTF16) {
        print!("UTF16 | ");
    }
}

fn read_buss_string(stream: &mut BufReader<TcpStream>, is_utf16: bool) -> io::Result<String> {
    let mut buf: [u8; 4] = [0; 4];
    stream.read_exact(&mut buf)?;
    let length = u32::from_be_bytes(buf);

    if length == 0 {
        return Ok(String::new());
    }
    let length = length as usize;

    let mut buf = vec![0; length];
    stream.read_exact(&mut buf)?;

    if !is_utf16 {
        Ok(String::from_utf8(buf).unwrap())
    } else {
        let iter = (0..length / 2).map(|i| u16::from_be_bytes([buf[2 * i], buf[2 * i + 1]]));
        let string = decode_utf16(iter).collect::<Result<String, _>>().unwrap();

        Ok(string)
    }
}

fn process_request(stream: &mut BufReader<TcpStream>) -> io::Result<()> {
    let mut header: [u8; 8] = [0; 8];
    stream.read_exact(&mut header)?;
    let header: BussHeader = BussHeader::from_bytes(&header);
    if header.magic_number != 0x00042069 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Not a bussin protocol header",
        ));
    }

    println!("[#] Request Header");
    println!("+-----------------------------------+");

    // Read the next 4 bytes for path
    let mut path_length: [u8; 4] = [0; 4];
    stream.read_exact(&mut path_length)?;
    let path_length = u32::from_be_bytes(path_length);
    // Read the next n bytes of "path length"
    let mut buff = vec![0; path_length as usize];
    stream.read_exact(&mut buff)?;
    // convert string to utf8
    let path = String::from_utf8_lossy(&buff);
    println!(
        " Bussin {}.{} {} {}",
        header.version_major,
        header.version_minor,
        action_to_string(header.action),
        path
    );
    print_flags(header.flags);

    let mut settings_count: [u8; 2] = [0; 2];
    stream.read_exact(&mut settings_count)?;
    let settings_count = u16::from_be_bytes(settings_count);
    println!(" Settings count: {}", settings_count);

    for i in 0..settings_count {
        // crunch through the settings
        let mut buf: [u8; 1] = [0; 1];
        stream.read_exact(&mut buf)?;
        let tag = u8::from_be_bytes(buf);
        if let Some(tag) = u8_to_settings(tag) {
            match tag {
                BussSettings::BodyLength => {
                    // read the next 4 bytes for the length
                    let mut buf: [u8; 4] = [0; 4];
                    stream.read_exact(&mut buf)?;
                    let length = u32::from_be_bytes(buf);
                    println!(" {}>BodyLength: {}", i, length);
                }
                BussSettings::Host => {
                    // read string
                    let is_utf16 = header.flags & UTF16 == UTF16;
                    let string = read_buss_string(stream, is_utf16)?;
                    println!(" {}>Host: {}", i, string);
                }
                BussSettings::Custom => {
                    // custom tag,
                    let length = read_u32(stream)? as usize;
                    println!(" {}>Custom: Length {}", i, length);
                    // skip length bytes
                    io::copy(&mut stream.by_ref().take(length as u64), &mut io::sink())?;
                }
            }
        } else {
            println!("Unknown settings {} ", tag);
        }
    }

    let mut buf = Vec::new();
    stream.read_to_end(&mut buf)?;

    println!(" Body: ");
    println!("{}", String::from_utf8_lossy(&buf));

    println!("+-----------------------------------+");
    Ok(())
}
