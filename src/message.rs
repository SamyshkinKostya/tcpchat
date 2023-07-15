use std::fmt::Display;

use colored::{Colorize, Color};

pub struct Message
{
    pub room: String,
    pub id: u16,
    pub nickname: String,
    pub color: Color,
    pub text: String,
}

impl Message
{
    pub fn new(room: String, id: u16, nickname: String, color: Color, text: String) -> Message
    {
        Message
        {
            room,
            id,
            nickname,
            color,
            text,
        }
    }
}

impl Display for Message
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        let nickname = format!("{}:", self.nickname).color(self.color).bold();
        let id = format!("[{}]", self.id).dimmed();
        writeln!(f, "{id} {nickname} {}", self.text)
    }
}