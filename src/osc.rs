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

pub fn send(host: &str, port: u16, address: &str, value: &str) -> Result<()> {
    let buf = encode_message(address, value)?;
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.send_to(&buf, (host, port))?;
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
