app commmands:
- kioto version
- kioto help
- kioto crate <roomname>
- kioto create <roomname> pass <password>
- kioto join <roomname>
- kioto join <roomname> pass <password>
- kioto join <roomname> as <username(color)>
- kioto join <roomname> as <username(color)> pass <password>
- kioto list
- kioto del <roomname>

chat keys:
- ctrl+x - quit
- shift+up/down - messages scrolling
- ctrl+shift+up/down - page scrolling

chat commands:
- /kick <username>

1. kioto version
kioto 0.1

2. kioto help
- kioto version - print version
- kioto help = print help
- kioto create <roomname> as <username>(<color>) - create room
- kioto join <roomname> as <username>(<color>) - join room
- kioto list - list rooms
- kioto del <roomname> - delete room

3. kioto create someroom pass dupa123
New room created.

4. kioto join someroom as someuser(green) pass dupa123
(chat screen with ctrl+x - quit underneath)

5. kioto list
(* - passworded)
- someroom <admin> *
- otherroom <guest>

6. kioto del someroom pass dupa123
Room deleted.