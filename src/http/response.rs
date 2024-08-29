use super::message::Startline;

pub struct StatusLine {
    version: String,
    code: u16,
    text: String
}

impl StatusLine {
    pub fn new(
        version: String,
        code: u16,
        text: String
    ) -> Self {
        Self {
            version,
            code,
            text
        }
    }
}

impl Startline for StatusLine {}

impl Into<String> for StatusLine {
    fn into(self) -> String {
        format!("{} {} {}", self.version, self.code, self.text)
    }
}