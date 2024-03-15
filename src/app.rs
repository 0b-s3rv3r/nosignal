use std::env;

pub enum CommandOption {
    Version,
    Help,
    Create {
        room_id: String,
        password: Option<String>,
    },
    NewJoin {
        room_address: String,
        username: String,
        password: Option<String>,
    },
    OldJoin {
        room_id: String,
        username: String,
        password: Option<String>,
    },
    Delete {
        room_id: String,
        password: Option<String>,
    },
    List,
}

pub struct CLI;

impl CLI {
    pub fn get_command_option() -> CommandOption {
        let args: Vec<String> = env::args().collect();

        let command = &args[1];
        match command.as_str() {
            "version" => CommandOption::Version,
            "help" => CommandOption::Help,
            "create" => {
                if args.len() < 3+1 {
                    eprintln!("Usage: kioto create <room_id>");
                    std::process::exit(1);
                }
                CommandOption::Create {
                    room_id: args[2].clone(),
                    password: Some(args[4].clone()),
                }
            }
            "join" => {
                if args.len() < 3+1 {
                    eprintln!("Usage: program_name join-room <room_address> [room_key]");
                    std::process::exit(1);
                }
                CommandOption::OldJoin { room_id: (), username: (), password: () }
                CommandOption::NewJoin { room_address: (), username: (), password: () }
            }
            "del" => {
                if args.len() < 3+1 {
                    eprintln!("Usage: kioto delete <room_id> pass <password>");
                    std::process::exit(1);
                }
                CommandOption::Delete {
                    room_id: args[2].clone(),
                    password: Some(args[3].clone())
                }
            }
            "roomlist" => {
                if args.len() < 2+1 {
                    eprintln!("Usage: kioto list");
                    std::process::exit(1);
                }
                CommandOption::List
            }
            _ => {
                eprintln!("Type 'kioto help' for more info");
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
            CommandOption::Version => todo!(),
            CommandOption::Help => todo!(),
            CommandOption::Create { room_id, password } => todo!(),
            CommandOption::NewJoin { room_address, username, password } => todo!(),
            CommandOption::OldJoin { room_id, username, password } => todo!(),
            CommandOption::Delete { room_id, password } => todo!(),
            CommandOption::List => todo!(),
        }
    }
}
