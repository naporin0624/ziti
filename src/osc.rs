use anyhow::Result;
use rosc::{encoder, OscMessage, OscPacket, OscType};
use std::net::UdpSocket;

pub fn encode_message(address: &str, value: &str) -> Result<Vec<u8>> {
    let packet = OscPacket::Message(OscMessage {
        addr: address.to_string(),
        args: vec![OscType::String(value.to_string())],
    });
    Ok(encoder::encode(&packet)?)
}

pub fn encode_float_message(address: &str, value: f32) -> Result<Vec<u8>> {
    let packet = OscPacket::Message(OscMessage {
        addr: address.to_string(),
        args: vec![OscType::Float(value)],
    });
    Ok(encoder::encode(&packet)?)
}

pub fn send(host: &str, port: u16, address: &str, value: &str) -> Result<()> {
    let buf = encode_message(address, value)?;
    send_packet(host, port, &buf)
}

pub fn send_float(host: &str, port: u16, address: &str, value: f32) -> Result<()> {
    let buf = encode_float_message(address, value)?;
    send_packet(host, port, &buf)
}

fn send_packet(host: &str, port: u16, buf: &[u8]) -> Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.send_to(buf, (host, port))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rosc::decoder;

    #[test]
    fn encodes_single_string_arg() {
        let buf = encode_message("/cannelloni/search", "Mirin Sheeno - Harmony").unwrap();
        let (_rest, packet) = decoder::decode_udp(&buf).unwrap();
        match packet {
            OscPacket::Message(m) => {
                assert_eq!(m.addr, "/cannelloni/search");
                assert_eq!(
                    m.args,
                    vec![OscType::String("Mirin Sheeno - Harmony".into())]
                );
            }
            _ => panic!("expected a message"),
        }
    }

    #[test]
    fn encodes_single_float_arg() {
        let buf = encode_float_message("/ziti/offset", 160.36).unwrap();
        let (_rest, packet) = decoder::decode_udp(&buf).unwrap();
        match packet {
            OscPacket::Message(m) => {
                assert_eq!(m.addr, "/ziti/offset");
                assert_eq!(m.args, vec![OscType::Float(160.36)]);
            }
            _ => panic!("expected a message"),
        }
    }

    #[test]
    fn send_float_delivers_over_udp() {
        let receiver = UdpSocket::bind("127.0.0.1:0").unwrap();
        let addr = receiver.local_addr().unwrap();

        send_float("127.0.0.1", addr.port(), "/ziti/offset", 88.5).unwrap();

        let mut buf = [0u8; 1024];
        let (n, _src) = receiver.recv_from(&mut buf).unwrap();
        let (_rest, packet) = decoder::decode_udp(&buf[..n]).unwrap();
        match packet {
            OscPacket::Message(m) => {
                assert_eq!(m.addr, "/ziti/offset");
                assert_eq!(m.args, vec![OscType::Float(88.5)]);
            }
            _ => panic!("expected a message"),
        }
    }

    #[test]
    fn send_delivers_over_udp() {
        let receiver = UdpSocket::bind("127.0.0.1:0").unwrap();
        let addr = receiver.local_addr().unwrap();

        send("127.0.0.1", addr.port(), "/cannelloni/search", "hello").unwrap();

        let mut buf = [0u8; 1024];
        let (n, _src) = receiver.recv_from(&mut buf).unwrap();
        let (_rest, packet) = decoder::decode_udp(&buf[..n]).unwrap();
        match packet {
            OscPacket::Message(m) => {
                assert_eq!(m.addr, "/cannelloni/search");
                assert_eq!(m.args, vec![OscType::String("hello".into())]);
            }
            _ => panic!("expected a message"),
        }
    }
}
