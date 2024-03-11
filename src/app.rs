use std::{env, path::Path};

pub enum CommandOption {
    Version,
    Help,
    CreateNewRoom {
        room_id: String,
    },
    JoinRoom {
        room_address: String,
        room_key: Option<String>,
    },
    DeleteRoom {
        room_id: String,
    },
    ListRooms,
    ListRoomUsers {
        room_id: String,
    },
}

pub struct CLI;

impl CLI {
    pub fn get_command_option() -> CommandOption {
        let args: Vec<String> = env::args().collect();

        if args.len() < 2 {
            eprintln!("Usage: program_name <command> [options]\nType chad -help for more info");
            std::process::exit(1);
        }

        let command = &args[1];
        match command.as_str() {
            "version" => CommandOption::Version,
            "help" => CommandOption::Help,
            "new" => {
                if args.len() < 3 {
                    eprintln!("Usage: program_name create-new-room <room_id>");
                    std::process::exit(1);
                }
                CommandOption::CreateNewRoom {
                    room_id: args[2].clone(),
                }
            }
            "join" => {
                if args.len() < 3 {
                    eprintln!("Usage: program_name join-room <room_address> [room_key]");
                    std::process::exit(1);
                }
                CommandOption::JoinRoom {
                    room_address: args[2].clone(),
                    room_key: args.get(3).cloned(),
                }
            }
            "del" => {
                if args.len() < 3 {
                    eprintln!("Usage: program_name delete-room <room_id>");
                    std::process::exit(1);
                }
                CommandOption::DeleteRoom {
                    room_id: args[2].clone(),
                }
            }
            "roomlist" => {
                if args.len() < 3 {
                    eprintln!("Usage: program_name list-room-users <room_id>");
                    std::process::exit(1);
                }
                CommandOption::ListRooms
            }
            "userlist" => {
                if args.len() < 3 {
                    eprintln!("Usage: program_name list-room-users <room_id>");
                    std::process::exit(1);
                }
                CommandOption::ListRoomUsers {
                    room_id: args[2].clone(),
                }
            }
            _ => {
                eprintln!(
                    "Invalid command: {}\nType 'chad help' for more info",
                    command
                );
                std::process::exit(1);
            }
        }
    }
}

pub struct App {
    cli: CLI,
}

impl App {
    pub fn run() {
        let options = CLI::get_command_option();
        match options {
            CommandOption::Version => println!("gigachat 0.1"),
            CommandOption::Help => println!(
            "Usage: program_name <command> [options]\n
            version - display version of gigachad\n
            help - display all availible options with description\n
            new [room_name] - create new chat room\n
            join [address] - join chat room\n
            del [room_name] - delete chat room\n
            roomlist - list chat rooms\n
            userlist [room_name] - list all users of chat room\n
            "),
            CommandOption::CreateNewRoom { room_id } => todo!(),
            CommandOption::JoinRoom {
                room_address,
                room_key,
            } => todo!(),
            CommandOption::DeleteRoom { room_id } => todo!(),
            CommandOption::ListRooms => todo!(),
            CommandOption::ListRoomUsers { room_id } => todo!(),
        }
    }
}