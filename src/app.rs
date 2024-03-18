use std::env;

pub enum CommandRequest {
    Version,
    Help,
    Create {
        room_id: String,
        has_password: bool,
    },
    Join {
        room_address: String,
        username: Option<String>,
    },
    Delete {
        room_id: String,
    },
    List,
    Invalid,
}

pub fn get_command_request() -> CommandRequest {
    let args: Vec<String> = env::args().collect();
    let len = args.len();

    if len < 2 {
        return CommandRequest::Invalid;
    }

    let command = &args[1];
    match command.as_str() {
        "version" => CommandRequest::Version,
        "help" => CommandRequest::Help,
        "create" => match len {
            3 => CommandRequest::Create {
                room_id: args[2].clone(),
                has_password: true,
            },
            4 if args[2] == "-n" => CommandRequest::Create {
                room_id: args[3].clone(),
                has_password: false,
            },
            _ => CommandRequest::Invalid,
        },
        "join" => match len {
            3 => CommandRequest::Join {
                room_address: args[2].clone(),
                username: None,
            },
            4 => CommandRequest::Join {
                room_address: args[2].clone(),
                username: Some(args[3].clone()),
            },
            _ => CommandRequest::Invalid,
        },
        "list" => CommandRequest::List,
        "del" => match len {
            3 => CommandRequest::Delete {
                room_id: args[2].clone(),
            },
            _ => CommandRequest::Invalid,
        },
        _ => CommandRequest::Invalid,
    }
}

pub struct App {}

impl App {
    pub fn run() {
        let options = get_command_request();
        match options {
            CommandRequest::Version => println!("yes"),
            CommandRequest::Help => println!("yes"),
            CommandRequest::Create {
                room_id,
                has_password,
            } => println!("yes"),
            CommandRequest::Join {
                room_address,
                username,
            } => println!("yes"),
            CommandRequest::Delete { room_id } => println!("yes"),
            CommandRequest::List => println!("yes"),
            CommandRequest::Invalid => println!("shit!"),
        }
    }
}
