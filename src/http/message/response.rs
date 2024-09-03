use super::message::{HttpMessage, Startline, Version};
use num_enum::{IntoPrimitive, TryFromPrimitive};
pub type Response = HttpMessage<StatusLine>;

#[derive(Clone, Copy)]
pub enum Status {
    Successful(Successful),
    ClientError(ClientError),
    ServerError(ServerError),
}

impl Into<u16> for Status {
    // TODO: when turning this into a library we
    // have to implement try_into (to ensure we don't crash if we forget codes?)
    // have to implement checked_add (in order to ensure we don't overflow)
    // do we really though? this is all internally controlled code.. ?

    fn into(self) -> u16 {
        match self {
            Status::Successful(s) => 200 + Into::<u8>::into(s) as u16,
            Status::ClientError(c) => 400 + Into::<u8>::into(c) as u16,
            Status::ServerError(s) => 500 + Into::<u8>::into(s) as u16,
        }
    }
}

impl Into<String> for Status {
    fn into(self) -> String {
        match self {
            Status::Successful(s) => match s {
                Successful::Ok => "OK".to_string(),
                Successful::Created => "Created".to_string(),
            },
            Status::ClientError(c) => match c {
                ClientError::NotFound => "Not Found".to_string(),
            },
            Status::ServerError(s) => match s {
                ServerError::Internal => "Internal Server Error".to_string(),
            },
        }
    }
}

#[derive(Debug, Eq, PartialEq, IntoPrimitive, TryFromPrimitive, Clone, Copy)]
#[repr(u8)]
pub enum Successful {
    Ok = 0,
    Created = 1,
}

#[derive(Debug, Eq, PartialEq, IntoPrimitive, TryFromPrimitive, Clone, Copy)]
#[repr(u8)]
pub enum ServerError {
    Internal = 0,
}

#[derive(Debug, Eq, PartialEq, IntoPrimitive, TryFromPrimitive, Clone, Copy)]
#[repr(u8)]
pub enum ClientError {
    NotFound = 4,
}

pub struct StatusLine {
    version: Version,
    status: Status,
}

impl StatusLine {
    pub fn new(version: Version, status: Status) -> Self {
        Self { version, status }
    }
}

impl Startline for StatusLine {}

impl Into<String> for StatusLine {
    fn into(self) -> String {
        let code: u16 = self.status.into();
        let message: String = self.status.into();
        let version: String = self.version.into();

        format!("{} {} {}", version, code, message)
    }
}
