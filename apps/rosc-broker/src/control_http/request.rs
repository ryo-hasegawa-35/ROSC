use std::io;

use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct HttpRequest {
    pub method: String,
    pub path: String,
}

pub(crate) async fn read_http_request(stream: &mut TcpStream) -> io::Result<HttpRequest> {
    let mut buffer = Vec::new();
    let mut chunk = [0u8; 1024];

    loop {
        let read = stream.read(&mut chunk).await?;
        if read == 0 {
            break;
        }
        buffer.extend_from_slice(&chunk[..read]);
        if buffer.windows(4).any(|window| window == b"\r\n\r\n") {
            break;
        }
        if buffer.len() > 8192 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "request headers exceed 8192 bytes",
            ));
        }
    }

    let request = String::from_utf8_lossy(&buffer);
    let Some(request_line) = request.lines().next() else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "missing request line",
        ));
    };

    let mut parts = request_line.split_whitespace();
    let Some(method) = parts.next() else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "missing request method",
        ));
    };
    let Some(path) = parts.next() else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "missing request path",
        ));
    };

    Ok(HttpRequest {
        method: method.to_owned(),
        path: path.to_owned(),
    })
}

pub(crate) fn split_query(path: &str) -> (&str, Option<&str>) {
    match path.split_once('?') {
        Some((path, query)) => (path, Some(query)),
        None => (path, None),
    }
}

pub(crate) fn replay_limit(query: Option<&str>) -> Result<usize, ()> {
    let Some(value) = query_parameter(query, "limit") else {
        return Ok(100);
    };

    let limit = value.parse::<usize>().map_err(|_| ())?;
    if limit == 0 {
        return Err(());
    }

    Ok(limit)
}

pub(crate) fn history_limit(query: Option<&str>) -> Result<Option<usize>, ()> {
    let Some(value) = query_parameter(query, "limit") else {
        return Ok(None);
    };

    let limit = value.parse::<usize>().map_err(|_| ())?;
    if limit == 0 {
        return Err(());
    }

    Ok(Some(limit))
}

pub(crate) fn allow_degraded(query: Option<&str>) -> Result<bool, ()> {
    match query_parameter(query, "allow_degraded") {
        None => Ok(false),
        Some("true" | "1") => Ok(true),
        Some("false" | "0") => Ok(false),
        Some(_) => Err(()),
    }
}

pub(crate) fn query_parameter<'a>(query: Option<&'a str>, key: &str) -> Option<&'a str> {
    query.and_then(|query| {
        query.split('&').find_map(|pair| {
            let (parameter_key, value) = pair.split_once('=')?;
            (parameter_key == key).then_some(value)
        })
    })
}

pub(crate) fn decode_uri_component(component: &str) -> Result<String, ()> {
    let bytes = component.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0usize;

    while index < bytes.len() {
        match bytes[index] {
            b'%' => {
                if index + 2 >= bytes.len() {
                    return Err(());
                }
                let high = decode_hex_nibble(bytes[index + 1]).ok_or(())?;
                let low = decode_hex_nibble(bytes[index + 2]).ok_or(())?;
                decoded.push((high << 4) | low);
                index += 3;
            }
            byte => {
                decoded.push(byte);
                index += 1;
            }
        }
    }

    String::from_utf8(decoded).map_err(|_| ())
}

fn decode_hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}
