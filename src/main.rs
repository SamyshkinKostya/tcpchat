use core::time;
use anyhow::Result;
use colored::Color;
use std::{
    sync::mpsc::channel,
    thread,
    io,
    collections::VecDeque
};

mod client;
use client::{Client, send, recv, ClientCommand};

mod message;
use message::Message;

mod room;
use room::Room;

mod server;
use server::Server;

mod tcp;
use tcp::ConnectionManager;

fn server_message(text: &str) -> String
{
    Message::new(
        "".to_string(),
        0,
        "[ Server ]".to_string(),
        Color::Red,
        text.to_string()
    ).to_string()
}

fn join_strings(hist: &VecDeque<String>) -> String
{
    let mut output = String::new();

    for i in hist
    {
        output.push_str(&i);
    }

    output
}

fn handle_commands(text: &String, serv: &mut Server, id: u16)
{
    let room = serv.rooms.get_mut(&serv.id_to_room[&id]).unwrap();
    let clients = &mut room.clients;
    let client = clients.get_mut(&id).unwrap();

    let args: Vec<&str> = text.split_whitespace().collect();
    match args[0]
    {
        "/nick" =>
        {
            if args.len() != 2
            {
                client.send(server_message("Usage: /nick <nickname>")).unwrap();
                return;
            }

            client.nickname = args[1].to_string();
            client.send(server_message(&format!("Set nickname to: {}", args[1]))).unwrap();
        },
        "/color" =>
        {
            if args.len() != 2
            {
                client.send(server_message("Usage: /color <color>")).unwrap();
                return;
            }

            client.color = args[1].parse().unwrap_or(client.color);
            client.send(server_message(&format!("Set color to {}", args[1]))).unwrap();
        },
        "/room" =>
        {
            if args.len() != 2
            {
                client.send(server_message("Usage: /room <room>")).unwrap();
                return;
            }

            let id = client.id;
            let room_name = args[1].to_string();
            let mut client = clients.remove(&id).unwrap();

            serv.rooms.entry(room_name.clone()).or_insert_with(|| Room::new(room_name.clone()));
            serv.id_to_room.insert(id, room_name.clone());

            // ESC[2J = clear screen, ESC[H = move cursor to 0, 0
            client.room = room_name.clone();
            client.send("\x1B[2J\x1B[H".to_string()).unwrap();
            client.send(join_strings(&serv.rooms[&room_name].history)).unwrap();
            serv.rooms.get_mut(&client.room).unwrap().clients.insert(client.id, client);
        },
        _ =>
        {
            client.send(server_message("Invalid command")).unwrap();
        },
    }
}

enum Command
{
    NewClient(Client),
    Msg(u16, String),
    RawMsg(Message),
    Kick(u16),
}

fn main() -> Result<()>
{
    let (tx, rx) = channel::<Command>();

    thread::spawn(move ||
    {
        let mut serv = Server::new();
        serv.rooms.insert("main".to_string(), Room::new("main".to_string()));

        loop
        {
            match rx.recv().unwrap()
            {
                Command::NewClient(client) =>
                {
                    client.send(join_strings(&serv.rooms["main"].history)).unwrap();
                    serv.id_to_room.insert(client.id, "main".to_string());
                    serv.rooms.get_mut("main").unwrap().clients.insert(client.id, client);
                },
                Command::Msg(id, text) =>
                {
                    // chat commands
                    if text.starts_with('/')
                    {
                        handle_commands(&text, &mut serv, id);

                        continue;
                    }

                    let room = serv.rooms.get_mut(&serv.id_to_room[&id]).unwrap();
                    let clients = &mut room.clients;
                    let client = clients.get_mut(&id).unwrap();

                    let msg = Message::new(
                        room.name.to_owned(),
                        id,
                        client.nickname.to_owned(),
                        client.color,
                        text
                    );

                    room.history.push_back(msg.to_string());
                    if room.history.len() > 15
                    {
                        room.history.pop_front();
                    }

                    for (id, client) in room.clients.iter_mut().filter(|(_, c)| c.id != msg.id)
                    {
                        if client.send(msg.to_string()).is_err()
                        {
                            serv.to_remove.push_back((room.name.to_owned(), *id));
                        }
                    }
                },
                Command::RawMsg(msg) =>
                {
                    for (id, client) in serv.rooms.get_mut(&msg.room).unwrap()
                        .clients
                        .iter_mut()
                        .filter(|(_, c)| c.id != msg.id)
                    {
                        if client.send(msg.to_string()).is_err()
                        {
                            serv.to_remove.push_back((msg.room.clone(), *id));
                        }
                    }
                },
                Command::Kick(id) =>
                {
                    let room = &serv.id_to_room[&id];
                    serv.rooms.get_mut(room).unwrap().clients.remove(&id).unwrap().kick().unwrap();
                },
            }

            while let Some((room, id)) = serv.to_remove.pop_back()
            {
                serv.rooms.get_mut(&room).unwrap().clients.remove(&id);
            }
        }
    });

    let tx_cmd = tx.clone();
    thread::spawn(move ||
    {
        for line in io::stdin().lines().flatten()
        {
            let args: Vec<&str> = line.split_whitespace().collect();

            if args.is_empty()
            {
                continue;
            }

            match args[0]
            {
                "kick" =>
                {
                    if args.len() != 2
                    {
                        println!("Usage: kick <id>");
                        continue;
                    }

                    let Ok(id) = args[1].parse::<u16>() else
                    {
                        println!("<id> is not a number");
                        continue;
                    };

                    tx_cmd.send(Command::Kick(id)).unwrap();
                },
                _ =>
                {
                    println!("Invalid command");
                },
            }
        }
    });

    let mgr = ConnectionManager::new().unwrap();
    let listener = mgr.bind("10.0.0.3", 8080);

    for (id, mut tcp) in listener.enumerate()
    {
        let (my_tx, my_rx) = channel();
        let mut client = Client::new("main".to_string(), id as u16 + 1, my_tx)?;

        let tx_client = tx.clone();
        thread::spawn(move ||
        {
            // ESC[2J = clear screen, ESC[H = move cursor to 0, 0
            send(&mut tcp, "\x1B[2J\x1B[H".to_string()).unwrap();
            send(&mut tcp, "Welcome to the chat! Please log in\n\nNickname: ".to_string()).unwrap();
            let client_id = client.id;

            loop
            {
                let Ok(text) = recv(&mut tcp) else
                {
                    thread::sleep(time::Duration::from_millis(100));
                    continue;
                };

                if text.is_empty()
                {
                    continue;
                }

                client.nickname = text;

                // ESC[2J = clear screen, ESC[H = move cursor to 0, 0
                send(&mut tcp, "\x1B[2J\x1B[H".to_string()).unwrap();

                tx_client.send(Command::RawMsg(Message::new(
                    client.room.clone(),
                    0,
                    "[ Server ]".to_string(),
                    Color::Red,
                    format!("{} joined", client.nickname)
                ))).unwrap();

                tx_client.send(Command::NewClient(client)).unwrap();
                break;
            }

            loop
            {
                let cmd = my_rx.try_recv();
                if let Ok(cmd) = cmd
                {
                    match cmd
                    {
                        ClientCommand::Msg(data) =>
                        {
                            if send(&mut tcp, data).is_err()
                            {
                                break;
                            }
                        },
                        ClientCommand::Kick =>
                        {
                            // ESC[2J = clear screen, ESC[H = move cursor to 0, 0
                            send(&mut tcp, "\x1B[2J\x1B[HKicked by an admin.".to_string()).unwrap();
                            break;
                        },
                    }
                }

                let Ok(text) = recv(&mut tcp) else
                {
                    thread::sleep(time::Duration::from_millis(100));
                    continue;
                };

                if text.is_empty()
                {
                    continue;
                }

                tx_client.send(Command::Msg(client_id, text)).unwrap();
            }
        });
    }

    Ok(())
}