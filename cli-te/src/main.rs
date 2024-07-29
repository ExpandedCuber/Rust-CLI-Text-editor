use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;
use std::time::Duration;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, ClearType},
    Result,
};

fn display_contents(content: &Vec<String>, current_line: usize, cursor_position: usize) -> Result<()> {
    execute!(io::stdout(), cursor::MoveTo(0, 0), crossterm::terminal::Clear(ClearType::All))?;
    for (i, line) in content.iter().enumerate() {
        if i == current_line {
            print!("> {}", line);
        } else {
            print!("  {}", line);
        }
        println!();
    }
    execute!(
        io::stdout(),
        cursor::MoveTo(
            cursor_position as u16 + 2,
            current_line as u16
        )
    )?;
    Ok(())
}

fn open_file(filename: &str) -> Result<()> {
    enable_raw_mode()?;
    let path = Path::new(filename);
    let mut content: Vec<String> = if path.exists() {
        let mut file = fs::File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        contents.lines().map(|line| line.to_string()).collect()
    } else {
        vec![String::new()]
    };

    let mut current_line = 0;
    let mut cursor_position = 0;

    display_contents(&content, current_line, cursor_position)?;

    loop {
        if let Ok(Event::Key(key_event)) = event::read() {
            if key_event.kind == KeyEventKind::Press {
                println!("Received key event {:?}", key_event);
                match (key_event.code, key_event.modifiers) {
                    (KeyCode::Up, _) => {
                        if current_line > 0 {
                            current_line -= 1;
                            cursor_position = cursor_position.min(content[current_line].len());
                        }
                    }
                    (KeyCode::Down, _) => {
                        if current_line < content.len() - 1 {
                            current_line += 1;
                            cursor_position = cursor_position.min(content[current_line].len());
                        }
                    }
                    (KeyCode::Left, _) => {
                        if cursor_position > 0 {
                            cursor_position -= 1;
                        } else if current_line > 0 {
                            current_line -= 1;
                            cursor_position = content[current_line].len();
                        }
                    }
                    (KeyCode::Right, _) => {
                        if cursor_position < content[current_line].len() {
                            cursor_position += 1;
                        } else if current_line < content.len() - 1 {
                            current_line += 1;
                            cursor_position = 0;
                        }
                    }
                    (KeyCode::Enter, _) => {
                        let remainder = content[current_line].split_off(cursor_position);
                        current_line += 1;
                        content.insert(current_line, remainder);
                        cursor_position = 0;
                    }
                    (KeyCode::Backspace, _) => {
                        if cursor_position > 0 {
                            content[current_line].remove(cursor_position - 1);
                            cursor_position -= 1;
                        } else if current_line > 0 {
                            let removed_line = content.remove(current_line);
                            current_line -= 1;
                            cursor_position = content[current_line].len();
                            content[current_line].push_str(&removed_line);
                        }
                    }
                    (KeyCode::Char('s'), KeyModifiers::CONTROL) => {
                        save_file(filename, &content)?;
                        println!("File saved successfully!");
                        disable_raw_mode()?;
                        break;
                    }
                    (KeyCode::Char(c), _) => {
                        content[current_line].insert(cursor_position, c);
                        cursor_position += 1;
                    }
                    (KeyCode::Esc, _) => break,
                    _ => {},
                }
            
                    display_contents(&content, current_line, cursor_position)?;
                }
            }
        }

    disable_raw_mode()?;
    Ok(())
}

fn save_file(filename: &str, lines: &Vec<String>) -> Result<()> {
    let content = lines.join("\n");
    fs::write(filename, content)?;
    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        println!("Usage: {} <filename>", args[0]);
        return Ok(());
    }

    let filename = &args[1];

    if let Err(e) = open_file(filename) {
        println!("Error: {}", e);
    }

    Ok(())
}

