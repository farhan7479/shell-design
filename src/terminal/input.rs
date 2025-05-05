use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;

/// Terminal input handler for keyboard events
pub struct InputHandler {
    last_key: Option<KeyEvent>,
}

/// Represents a terminal input event
pub enum InputEvent {
    /// Key press event
    Key(KeyEvent),
    /// Terminal resize event with new dimensions (width, height)
    Resize(u16, u16),
    /// No event available
    None,
}

impl InputHandler {
    /// Create a new input handler
    pub fn new() -> Self {
        Self { last_key: None }
    }
    
    /// Poll for input events with timeout
    pub fn poll_event(&mut self, timeout_ms: u64) -> Result<InputEvent> {
        // Check if there's an event available within the timeout period
        if event::poll(Duration::from_millis(timeout_ms))? {
            match event::read()? {
                Event::Key(key) => {
                    self.last_key = Some(key);
                    return Ok(InputEvent::Key(key));
                }
                Event::Resize(width, height) => {
                    return Ok(InputEvent::Resize(width, height));
                }
                _ => {}
            }
        }
        
        Ok(InputEvent::None)
    }
    
    /// Check if a specific key was pressed
    pub fn is_key_pressed(&self, code: KeyCode) -> bool {
        if let Some(key) = self.last_key {
            key.code == code
        } else {
            false
        }
    }
    
    /// Check if a key with specific modifiers was pressed
    pub fn is_key_with_modifier(&self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        if let Some(key) = self.last_key {
            key.code == code && key.modifiers == modifiers
        } else {
            false
        }
    }
    
    /// Process keyboard input and convert to appropriate byte sequence for PTY
    pub fn process_key_input(&self, key: KeyEvent) -> Vec<u8> {
        match key.code {
            KeyCode::Char(c) => {
                // Handle Ctrl+key combinations
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    // Convert to control character (ASCII control characters are 1-26)
                    if c >= 'a' && c <= 'z' {
                        return vec![(c as u8 - b'a' + 1)];
                    } else if c >= 'A' && c <= 'Z' {
                        return vec![(c as u8 - b'A' + 1)];
                    }
                }
                
                // Regular character input
                vec![c as u8]
            },
            KeyCode::Enter => vec![b'\r'],
            KeyCode::Tab => vec![b'\t'],
            KeyCode::Backspace => vec![0x7F], // DEL character
            KeyCode::Esc => vec![0x1B],       // ESC character
            
            // Use standard VT100 escape sequences for cursor keys
            KeyCode::Up => vec![0x1B, b'[', b'A'],
            KeyCode::Down => vec![0x1B, b'[', b'B'],
            KeyCode::Right => vec![0x1B, b'[', b'C'],
            KeyCode::Left => vec![0x1B, b'[', b'D'],
            
            // More standard escape sequences
            KeyCode::Home => vec![0x1B, b'[', b'H'],
            KeyCode::End => vec![0x1B, b'[', b'F'],
            KeyCode::PageUp => vec![0x1B, b'[', b'5', b'~'],
            KeyCode::PageDown => vec![0x1B, b'[', b'6', b'~'],
            KeyCode::Delete => vec![0x1B, b'[', b'3', b'~'],
            KeyCode::Insert => vec![0x1B, b'[', b'2', b'~'],
            
            // Function keys
            KeyCode::F(n) => {
                match n {
                    1 => vec![0x1B, b'O', b'P'],
                    2 => vec![0x1B, b'O', b'Q'],
                    3 => vec![0x1B, b'O', b'R'],
                    4 => vec![0x1B, b'O', b'S'],
                    5 => vec![0x1B, b'[', b'1', b'5', b'~'],
                    6 => vec![0x1B, b'[', b'1', b'7', b'~'],
                    7 => vec![0x1B, b'[', b'1', b'8', b'~'],
                    8 => vec![0x1B, b'[', b'1', b'9', b'~'],
                    9 => vec![0x1B, b'[', b'2', b'0', b'~'],
                    10 => vec![0x1B, b'[', b'2', b'1', b'~'],
                    11 => vec![0x1B, b'[', b'2', b'3', b'~'],
                    12 => vec![0x1B, b'[', b'2', b'4', b'~'],
                    _ => vec![], // Unknown function key
                }
            }
            _ => vec![], // Unhandled key
        }
    }
}