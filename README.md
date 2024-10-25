# 💬nosignal

Single-executable and minimalistic tui chat written with spaghetti code.

![Screenshot from 2024-10-20 23-06-15](https://github.com/user-attachments/assets/61401402-9823-4eed-a546-7d694ddaac21)

Build
```
cargo build --release
```

Run tests
```
cargo test -- --test-threads=1
```

Commands
```
create, -c, --create  Creates a new room
join, -j, --join      Joins a room
delete, -d, --delete  Deletes a room
list, -l, --list      Lists all rooms
set, -s, --set        Sets an application option
help                  Print this message or the help of the given subcommand(s)
```

User colors available
- black
- red
- green
- yellow
- blue
- magenta
- cyan
- gray
- darkgray
- lightred
- lightgreen
- lightyellow
- lightblue
- lightmagenta
- lightcyan
- white
