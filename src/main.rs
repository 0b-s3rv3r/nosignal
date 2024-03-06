use std::{env, path::Path, sync::Condvar};

pub enum CommandOption {
    Version,
    Help,
    CreateNewUser {
        username: String,
        color: String,
    },
    DeleteUser {
        username: String,
    },
    ListLocalUsers,
    ChangeUsername {
        username: String,
        new_username: String,
    },
    ChangePassword {
        username: String,
        new_password: String,
    },
    ChangeColor {
        username: String,
        color: String,
    },
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
    ListRoomUsers {
        room_id: String,
    },
    RemoveRoomUser {
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
            "-version" => CommandOption::Version,
            "-help" => CommandOption::Help,
            "-usernew" => {
                if args.len() < 4 {
                    eprintln!("Usage: program_name create-new-user <username> <password>");
                    std::process::exit(1);
                }
                CommandOption::CreateNewUser {
                    username: args[2].clone(),
                    color: args[3].clone(),
                }
            }
            "userdel" => {
                if args.len() < 3 {
                    eprintln!("Usage: program_name delete-user <username>");
                    std::process::exit(1);
                }
                CommandOption::DeleteUser {
                    username: args[2].clone(),
                }
            }
            "userlist" => CommandOption::ListLocalUsers,
            "namechange" => {
                if args.len() < 4 {
                    eprintln!("Usage: program_name change-username <username> <new_username>");
                    std::process::exit(1);
                }
                CommandOption::ChangeUsername {
                    username: args[2].clone(),
                    new_username: args[3].clone(),
                }
            }
            "passwordchange" => {
                if args.len() < 4 {
                    eprintln!("Usage: program_name change-password <username> <new_password>");
                    std::process::exit(1);
                }
                CommandOption::ChangePassword {
                    username: args[2].clone(),
                    new_password: args[3].clone(),
                }
            }
            "colorchange" => {
                if args.len() < 4 {
                    eprintln!("Usage: program_name change-password <username> <new_password>");
                    std::process::exit(1);
                }
                CommandOption::ChangeColor {
                    username: args[2].clone(),
                    color: args[3].clone(),
                }
            }
            "roomnew" => {
                if args.len() < 3 {
                    eprintln!("Usage: program_name create-new-room <room_id>");
                    std::process::exit(1);
                }
                CommandOption::CreateNewRoom {
                    room_id: args[2].clone(),
                }
            }
            "roomjoin" => {
                if args.len() < 3 {
                    eprintln!("Usage: program_name join-room <room_address> [room_key]");
                    std::process::exit(1);
                }
                CommandOption::JoinRoom {
                    room_address: args[2].clone(),
                    room_key: args.get(3).cloned(),
                }
            }
            "roomdelete" => {
                if args.len() < 3 {
                    eprintln!("Usage: program_name delete-room <room_id>");
                    std::process::exit(1);
                }
                CommandOption::DeleteRoom {
                    room_id: args[2].clone(),
                }
            }
            "roomuserlist" => {
                if args.len() < 3 {
                    eprintln!("Usage: program_name list-room-users <room_id>");
                    std::process::exit(1);
                }
                CommandOption::ListRoomUsers {
                    room_id: args[2].clone(),
                }
            }
            "roomuserdelete" => {
                if args.len() < 3 {
                    eprintln!("Usage: program_name remove-room-user <room_id>");
                    std::process::exit(1);
                }
                CommandOption::RemoveRoomUser {
                    room_id: args[2].clone(),
                }
            }
            _ => {
                eprintln!("Invalid command: {}", command);
                std::process::exit(1);
            }
        }
    }
}

enum Role {
    Host,
    Guest,
}

pub struct User {
    user_id: usize,
    username: String,
    password: String,
}

pub struct UserManager {
    filepath: Path,
}

impl UserManager {
    pub fn get_user(username: String) {}
    pub fn get_all_users() {}
    pub fn add_user(user: User) {}
    pub fn update_user(user: User) {}
    pub fn delete_user(user: User) {}
}

struct ChatRoom {
    room_id: String,
    address: String,
    password: Option<String>,
    host: User,
    guests: Vec<User>,
}

impl ChatRoom {
    pub fn add_message(content: String, sender: User) {}
    pub fn add_guest(guest: User) {}
    pub fn remove_guest(guest: User) {}
}

struct ChatDb {
    filepath: Path,
}

impl ChatDb {
    pub fn get_chatroom(room_id: String) {}
    pub fn add_chatroom(room: ChatRoom) {}
    pub fn update_chatroom(room: ChatRoom) {}
    pub fn delete_chatroom(room: ChatRoom) {}
}

struct App {
    cli: CLI,
}

impl App {
    pub fn run() {
        let options = CLI::get_command_option();
        match options {
            CommandOption::Version => todo!(),
            CommandOption::Help => todo!(),
            CommandOption::CreateNewUser { username, color } => todo!(),
            CommandOption::DeleteUser { username } => todo!(),
            CommandOption::ListLocalUsers => todo!(),
            CommandOption::ChangeUsername {
                username,
                new_username,
            } => todo!(),
            CommandOption::ChangePassword {
                username,
                new_password,
            } => todo!(),
            CommandOption::ChangeColor {
                username,
                color,
            } => todo!(),
            CommandOption::CreateNewRoom { room_id } => todo!(),
            CommandOption::JoinRoom {
                room_address,
                room_key,
            } => todo!(),
            CommandOption::DeleteRoom { room_id } => todo!(),
            CommandOption::ListRoomUsers { room_id } => todo!(),
            CommandOption::RemoveRoomUser { room_id } => todo!(),
        }
    }
}

fn main() {
    println!("Hello, world!");
}
