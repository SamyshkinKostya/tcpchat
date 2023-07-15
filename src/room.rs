use std::collections::{HashMap, VecDeque};

use crate::client::Client;

pub struct Room
{
    pub name: String,
    pub history: VecDeque<String>,
    pub clients: HashMap::<u16, Client>,
}

impl Room
{
    pub fn new(name: String) -> Room
    {
        Room
        {
            name,
            history: VecDeque::new(),
            clients: HashMap::new(),
        }
    }
}