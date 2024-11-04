# Sage
![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)

Sage is a small console-based text editor inspired by Vim.

<img src="https://github.com/user-attachments/assets/c6dbcc26-4ee6-421c-a2a8-3a00555b19ef" width="50%" height="150"/>

### Installation and Usage
Clone the repository:
```
git clone https://github.com/lucasmolinari/sage
cd sage
```
Build and run the application:
```
cargo run --release
```
You can also provide a file to open and edit:
```
cargo run --release src/main.rs
```


### Movement
Cursor Movement in Normal mode.
| Keys     | Motion                                    |
| -------- | ----------------------------------------- |
| h        | Move cursor left                          |
| j        | Move cursor down                          |
| k        | Move cursor up                            |
| l        | Move cursor right                         |
| w        | Move forward to the start of a word       |
| e        | Move forward to the end of a word         |
| b        | Move backward to the start of a word      |
| ge       | Move backward to the end of a word        |
| _        | Move to the start of the line             |
| $        | Move to the end of the line               |
| gg       | Move to the first line                    |
| G        | Move to the last line                     |


### Editing
Enter Insert Mode to directly write to the file, similar to Nano.
| Keys |  Action                             | Mode          |
| -----| ----------------------------------- | ------------- | 
| x    | Delete character at cursor position | Normal        | 
| dd   | Delete current line                 | Normal        |
| a    | Append                              | Insert        |
| i    | Prepend                             | Insert        |
| I    | Move to start of the line           | Insert        |
| A    | Move to end of the line             | Insert        |
| o    | Insert new line bellow cursor       | Insert        |
| O    | Insert new line above cursor        | Insert        |
| Esc  | Leave Insert Mode                   | Insert        |


### Commands
Use `Esc` to leave command mode and `Enter` to execute current command.
| Command | Optional Args         | Action                           |
| ------- | -------------------- | --------------------------------- | 
| :w      | New file name        | Write file                        |
| :q      | !                    | Quit editor - ! to ignore changes |
| :wq     | New file name        | Write file and quit               |



Based on the following source codes: 
- [Kilo](https://github.com/antirez/kilo)
- [Pound](https://github.com/Kofituo/pound)
