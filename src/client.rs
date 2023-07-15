use std::sync::mpsc::Sender;
use anyhow::Result;
use colored::Color;

use crate::tcp::ConnectionHandle;

pub enum ClientCommand
{
    Msg(String),
    Kick,
}

pub struct Client
{
    pub room: String,
    pub id: u16,
    pub tx: Sender<ClientCommand>,
    pub nickname: String,
    pub color: Color,
    pub to_remove: bool,
}

impl Client
{
    pub fn new(room: String, id: u16, tx: Sender<ClientCommand>) -> Result<Client>
    {
        Ok(Client
        {
            room,
            id,
            tx,
            nickname: "".to_string(),
            color: Color::Green,
            to_remove: false,
        })
    }

    pub fn send(&self, data: String) -> Result<()>
    {
        self.tx.send(ClientCommand::Msg(data))?;
        Ok(())
    }

    pub fn kick(&self) -> Result<()>
    {
        self.tx.send(ClientCommand::Kick)?;
        Ok(())
    }
}

pub fn recv(stream: &mut ConnectionHandle) -> Result<String>
{
    let mut buf = vec![0; 1024];
    let _ = stream.read(&mut buf);

    let buf = strip_ansi_escapes::strip(buf)?;
    let text = &String::from_utf8(buf)?;

    Ok(text.trim().replace('\n', ""))
}

pub fn send(stream: &mut ConnectionHandle, data: String) -> Result<()>
{
    stream.write_all(data.as_bytes().to_owned());
    Ok(())
}