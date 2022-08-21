use core::str::from_utf8;

use embedded_hal::serial::{Read, Write};
use longan_nano::sprintln;
use nb::block;

use self::at::Response;

mod coap;

pub struct EC01F<TX, RX> {
    tx: TX,
    rx: RX,
}

pub enum Error<TX: Write<u8>, RX: Read<u8>> {
    TxError(TX::Error),
    RxError(RX::Error),
    Timeout,     // 响应超时
    BufferFull,  // 响应大小过长，缓冲区容量不足
    InvalidUTF8, // 收到的响应不是合法的UTF-8字符串
    ParseErr,    // 响应解析错误
}

impl<TX, RX> EC01F<TX, RX>
where
    TX: Write<u8>,
    RX: Read<u8>,
{
    pub fn new(tx: TX, rx: RX) -> nb::Result<Self, Error<TX, RX>> {
        let mut ec01f = Self { tx, rx };
        ec01f.check()?;
        Ok(ec01f)
    }

    fn check(&mut self) -> Result<(), Error<TX, RX>> {
        self.write_cmd("AT")?;
        while !matches!(self.read_resp()?, Response::Ok) {}

        Ok(())
        // self.write_cmd("ATQ1") // 抑制PING/IPERF/LWM2M主动上报的内容
    }

    fn write_cmd(&mut self, s: &str) -> Result<(), Error<TX, RX>> {
        for byte in s.as_bytes().iter().chain(b"\r") {
            block!(self.tx.write(*byte)).map_err(|e| Error::TxError(e))?;
        }
        Ok(())
    }

    fn read_byte(&mut self) -> Result<u8, Error<TX, RX>> {
        block!(self.rx.read()).map_err(|e| Error::RxError(e))
    }

    fn read_line<const N: usize>(
        &mut self,
        buffer: &mut heapless::Vec<u8, N>,
    ) -> Result<(), Error<TX, RX>> {
        loop {
            let c = self.read_byte()?;
            if let Err(_c) = buffer.push(c) {
                return Err(Error::BufferFull);
            }
            if c == b'\n' {
                return Ok(());
            }
        }
    }

    fn read_resp(&mut self) -> Result<Response, Error<TX, RX>> {
        let mut buffer = heapless::Vec::<u8, 1024>::new();
        loop {
            self.read_line(&mut buffer)?;

            let s = from_utf8(buffer.as_slice()).map_err(|_| Error::InvalidUTF8)?;
            match at::response(s) {
                Ok((_, Response::Empty)) => buffer.clear(),
                Ok((_s, resp)) => return Ok(resp),
                Err(e) => {
                    sprintln!("Parse error: {}", e);
                    return Err(Error::ParseErr);
                }
            };
        }
    }

    fn skip_line(&mut self) -> Result<(), RX::Error> {
        while block!(self.rx.read())? != b'\n' {}
        Ok(())
    }
}

mod at {
    use nom::{
        branch::alt,
        bytes::complete::tag,
        character::complete::digit1,
        combinator::{map, map_res, value},
        sequence::tuple,
        IResult,
    };

    #[derive(Clone)]
    pub enum Response {
        Empty, // <CR><LF>
        Ok,
        Error,
        CmeError(i32),
    }

    fn empty(input: &str) -> IResult<&str, Response> {
        value(Response::Empty, tag("\r\n"))(input)
    }

    fn ok(input: &str) -> IResult<&str, Response> {
        value(Response::Ok, tag("\r\nOK\r\n"))(input)
    }

    fn error(input: &str) -> IResult<&str, Response> {
        value(Response::Error, tag("\r\nERROR\r\n"))(input)
    }

    fn cme_error(input: &str) -> IResult<&str, Response> {
        map(
            tuple((
                tag("\r\n+CME ERROR: "),
                map_res(digit1, |v: &str| v.parse()),
                tag("\r\n"),
            )),
            |(_, v, _)| Response::CmeError(v),
        )(input)
    }

    pub fn response(input: &str) -> IResult<&str, Response> {
        alt((ok, error, cme_error, empty))(input)
    }
}
