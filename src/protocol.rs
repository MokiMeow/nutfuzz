use std::io::{self, Read, Write};

use crate::exec::RunResult;

pub const PROTOCOL_VERSION: u32 = 1;
pub const MAX_FRAME_SIZE: usize = 64 * 1024;

const HELLO: u8 = 1;
const RUN: u8 = 2;
const REPLY: u8 = 3;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ForkserverReply {
    pub result: RunResult,
    pub coverage: Vec<(usize, u8)>,
}

pub fn write_hello(writer: &mut impl Write) -> io::Result<()> {
    write_frame(writer, &message(HELLO, &[]))
}

pub fn read_hello(reader: &mut impl Read) -> io::Result<()> {
    let frame = read_frame(reader)?;
    let (_, body) = parse_header(&frame, HELLO)?;
    if body.is_empty() {
        Ok(())
    } else {
        Err(invalid("hello frame has a body"))
    }
}

pub fn write_run(writer: &mut impl Write, input: &[u8]) -> io::Result<()> {
    write_frame(writer, &message(RUN, input))
}

pub fn read_run(reader: &mut impl Read) -> io::Result<Vec<u8>> {
    let frame = read_frame(reader)?;
    let (_, body) = parse_header(&frame, RUN)?;
    Ok(body.to_vec())
}

pub fn write_reply(
    writer: &mut impl Write,
    reply: &ForkserverReply,
) -> io::Result<()> {
    let mut body = Vec::new();
    let (kind, code, result_message) = match &reply.result {
        RunResult::Exited(code) => (0, *code, ""),
        RunResult::Crashed(signal, name) => (1, *signal, name.as_str()),
        RunResult::Hang => (2, 0, ""),
        RunResult::HarnessError(error) => (3, 0, error.as_str()),
    };
    body.push(kind);
    body.extend_from_slice(&code.to_le_bytes());
    let message_bytes = result_message.as_bytes();
    let message_len = u16::try_from(message_bytes.len())
        .map_err(|_| invalid("result message is too long"))?;
    body.extend_from_slice(&message_len.to_le_bytes());
    body.extend_from_slice(message_bytes);
    let coverage_len = u32::try_from(reply.coverage.len())
        .map_err(|_| invalid("coverage map is too large"))?;
    body.extend_from_slice(&coverage_len.to_le_bytes());
    for &(index, value) in &reply.coverage {
        let index =
            u32::try_from(index).map_err(|_| invalid("coverage index is too large"))?;
        body.extend_from_slice(&index.to_le_bytes());
        body.push(value);
    }
    write_frame(writer, &message(REPLY, &body))
}

pub fn read_reply(reader: &mut impl Read) -> io::Result<ForkserverReply> {
    let frame = read_frame(reader)?;
    let (_, mut body) = parse_header(&frame, REPLY)?;
    let kind = take_u8(&mut body)?;
    let code = take_i32(&mut body)?;
    let message_len = take_u16(&mut body)? as usize;
    let message_bytes = take(&mut body, message_len)?;
    let message = String::from_utf8(message_bytes.to_vec())
        .map_err(|_| invalid("result message is not UTF-8"))?;
    let coverage_len = take_u32(&mut body)? as usize;
    if coverage_len > MAX_FRAME_SIZE / 5 {
        return Err(invalid("coverage entry count is too large"));
    }
    let mut coverage = Vec::with_capacity(coverage_len);
    for _ in 0..coverage_len {
        coverage.push((take_u32(&mut body)? as usize, take_u8(&mut body)?));
    }
    if !body.is_empty() {
        return Err(invalid("reply frame has trailing bytes"));
    }
    let result = match kind {
        0 => RunResult::Exited(code),
        1 => RunResult::Crashed(code, message),
        2 => RunResult::Hang,
        3 => RunResult::HarnessError(message),
        _ => return Err(invalid("unknown result kind")),
    };
    Ok(ForkserverReply { result, coverage })
}

fn message(kind: u8, body: &[u8]) -> Vec<u8> {
    let mut message = Vec::with_capacity(5 + body.len());
    message.extend_from_slice(&PROTOCOL_VERSION.to_le_bytes());
    message.push(kind);
    message.extend_from_slice(body);
    message
}

fn parse_header(frame: &[u8], expected_kind: u8) -> io::Result<(u32, &[u8])> {
    if frame.len() < 5 {
        return Err(invalid("frame header is truncated"));
    }
    let version = u32::from_le_bytes(frame[..4].try_into().unwrap());
    if version != PROTOCOL_VERSION {
        return Err(invalid("unsupported protocol version"));
    }
    if frame[4] != expected_kind {
        return Err(invalid("unexpected frame kind"));
    }
    Ok((version, &frame[5..]))
}

fn write_frame(writer: &mut impl Write, payload: &[u8]) -> io::Result<()> {
    if payload.len() > MAX_FRAME_SIZE {
        return Err(invalid("frame is too large"));
    }
    let len = payload.len() as u32;
    writer.write_all(&len.to_le_bytes())?;
    writer.write_all(payload)?;
    writer.flush()
}

fn read_frame(reader: &mut impl Read) -> io::Result<Vec<u8>> {
    let mut length = [0u8; 4];
    reader.read_exact(&mut length)?;
    let length = u32::from_le_bytes(length) as usize;
    if length > MAX_FRAME_SIZE {
        return Err(invalid("frame is too large"));
    }
    let mut payload = vec![0; length];
    reader.read_exact(&mut payload)?;
    Ok(payload)
}

fn take<'a>(body: &mut &'a [u8], count: usize) -> io::Result<&'a [u8]> {
    if body.len() < count {
        return Err(invalid("reply frame is truncated"));
    }
    let (value, rest) = body.split_at(count);
    *body = rest;
    Ok(value)
}

fn take_u8(body: &mut &[u8]) -> io::Result<u8> {
    Ok(take(body, 1)?[0])
}

fn take_u16(body: &mut &[u8]) -> io::Result<u16> {
    Ok(u16::from_le_bytes(take(body, 2)?.try_into().unwrap()))
}

fn take_u32(body: &mut &[u8]) -> io::Result<u32> {
    Ok(u32::from_le_bytes(take(body, 4)?.try_into().unwrap()))
}

fn take_i32(body: &mut &[u8]) -> io::Result<i32> {
    Ok(i32::from_le_bytes(take(body, 4)?.try_into().unwrap()))
}

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
