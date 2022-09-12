use core::fmt::{self, Debug};
use core::str::from_utf8;
use embedded_hal::serial::{Read, Write};
use longan_nano::sprintln;
use nb::block;

use self::at::{Device, Response};

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

impl<TX: Write<u8>, RX: Read<u8>> Debug for Error<TX, RX> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TxError(_) => write!(f, "TxError"),
            Self::RxError(_) => write!(f, "RxError"),
            Self::Timeout => write!(f, "Timeout"),
            Self::BufferFull => write!(f, "BufferFull"),
            Self::InvalidUTF8 => write!(f, "InvalidUTF8"),
            Self::ParseErr => write!(f, "ParseErr"),
        }
    }
}

impl<TX, RX> EC01F<TX, RX>
where
    TX: Write<u8>,
    RX: Read<u8>,
{
    pub fn new(tx: TX, rx: RX) -> nb::Result<Self, Error<TX, RX>> {
        let mut ec01f = Self { tx, rx };
        ec01f.write_cmd(format_args!("ATE0"))?; // 禁用命令回显
        ec01f.read_ok()?;
        Ok(ec01f)
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

    pub fn create_coap(&mut self) -> coap::CoAPClient<'_, Self> {
        coap::CoAPClient::new(self)
    }
}

impl<TX, RX> fmt::Write for EC01F<TX, RX>
where
    TX: Write<u8>,
    RX: Read<u8>,
{
    fn write_str(&mut self, s: &str) -> fmt::Result {
        s.as_bytes()
            .iter()
            .try_for_each(|c| nb::block!(self.tx.write(*c)))
            .map_err(|_| core::fmt::Error)
    }
}

impl<TX, RX> at::Device for EC01F<TX, RX>
where
    TX: Write<u8>,
    RX: Read<u8>,
{
    type Error = Error<TX, RX>;

    fn write_cmd(&mut self, args: fmt::Arguments) -> Result<(), Error<TX, RX>> {
        fmt::Write::write_fmt(self, args).unwrap();
        fmt::Write::write_char(self, '\n').unwrap();
        Ok(())
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
}

mod at {
    use core::fmt;
    use nom::{
        branch::alt,
        bytes::complete::tag,
        character::complete::digit1,
        combinator::{map, map_res, value},
        sequence::tuple,
        IResult,
    };

    pub trait Device {
        type Error;
        fn write_cmd(&mut self, args: fmt::Arguments) -> Result<(), Self::Error>;
        fn read_resp(&mut self) -> Result<Response, Self::Error>;
        fn read_ok(&mut self) -> Result<(), Self::Error> {
            while !matches!(self.read_resp()?, Response::Ok) {}
            Ok(())
        }
    }

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
