use std::collections::{VecDeque, HashMap};

use crate::room::Room;

pub struct Server
{
    pub rooms: HashMap<String, Room>,
    pub id_to_room: HashMap<u16, String>,
    pub to_remove: VecDeque<(String, u16)>,
}

impl Server
{
    pub fn new() -> Server
    {
        Server
        {
            rooms: HashMap::new(),
            id_to_room: HashMap::new(),
            to_remove: VecDeque::new(),
        }
    }
}