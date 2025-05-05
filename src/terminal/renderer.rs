use anyhow::Result;
use chrono::Local;
use crossterm::{
    cursor,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{Clear, ClearType},
    ExecutableCommand,
};
use std::io::{stdout, Write};
use vt100::Parser;

/// Handles rendering of terminal content using VT100 parsing
pub struct TerminalRenderer {
    parser: Parser,
    width: u16,
    height: u16,
    title: String,
    show_status_bar: bool,
    raw_mode: bool,  // Add a flag to toggle raw mode for debugging
}

impl TerminalRenderer {
    /// Create a new terminal renderer with the specified dimensions
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            // Reserve rows for title and status bars
            parser: Parser::new(height.saturating_sub(2), width, 0),
            width,
            height,
            title: "Shell Terminal".to_string(),
            show_status_bar: true,
            raw_mode: false,
        }
    }
    
    /// Set the terminal title
    pub fn set_title(&mut self, title: String) {
        self.title = title;
    }
    
    /// Toggle the status bar display
    pub fn toggle_status_bar(&mut self) {
        self.show_status_bar = !self.show_status_bar;
        
        // Adjust parser height based on whether status bar is shown
        let reserved_rows = if self.show_status_bar { 2 } else { 1 };
        self.parser = Parser::new(self.height.saturating_sub(reserved_rows), self.width, 0);
    }
    
    /// Toggle raw mode for debugging escape sequences
    pub fn toggle_raw_mode(&mut self) {
        self.raw_mode = !self.raw_mode;
    }
    
    /// Process raw PTY output and update internal terminal state
    pub fn process_output(&mut self, data: &[u8]) {
        // Process the data through the VT100 parser
        self.parser.process(data);
        
        // Optionally log raw data for debugging
        if self.raw_mode {
            // Convert control characters to visible form for debugging
            let mut debug_str = String::new();
            for byte in data {
                match byte {
                    0..=31 | 127 => debug_str.push_str(&format!("^{}", (byte + 64) as char)),
                    _ => debug_str.push(*byte as char),
                }
            }
            eprintln!("Raw data: {}", debug_str);
        }
    }
    
    /// Resize the terminal
    pub fn resize(&mut self, width: u16, height: u16) {
        self.width = width;
        self.height = height;
        
        // Reserve rows for title and status bars
        let reserved_rows = if self.show_status_bar { 2 } else { 1 };
        self.parser = Parser::new(height.saturating_sub(reserved_rows), width, 0);
    }
    
    /// Render the title bar
    fn render_title_bar(&self) -> Result<()> {
        let mut stdout = stdout();
        
        // Move to the top of the screen
        stdout.execute(cursor::MoveTo(0, 0))?;
        
        // Set title bar colors (dark blue background with white text)
        stdout.execute(SetBackgroundColor(Color::DarkBlue))?;
        stdout.execute(SetForegroundColor(Color::White))?;
        
        // Clear the title bar line
        for _ in 0..self.width {
            stdout.execute(Print(" "))?;
        }
        
        // Move back to the start of the line and print the title
        stdout.execute(cursor::MoveTo(0, 0))?;
        
        // Format and print the title
        let centered_title = format!(" {} ", self.title);
        let position = (self.width as usize).saturating_sub(centered_title.len()) / 2;
        
        // Print spaces until the position
        for _ in 0..position {
            stdout.execute(Print(" "))?;
        }
        
        // Print the title
        stdout.execute(Print(centered_title))?;
        
        // Reset colors
        stdout.execute(ResetColor)?;
        
        Ok(())
    }
    
    /// Render the status bar at the bottom of the terminal
    fn render_status_bar(&self) -> Result<()> {
        if !self.show_status_bar {
            return Ok(());
        }
        
        let mut stdout = stdout();
        
        // Move to the bottom of the screen
        stdout.execute(cursor::MoveTo(0, self.height - 1))?;
        
        // Set status bar colors (dark gray background with light text)
        stdout.execute(SetBackgroundColor(Color::DarkGrey))?;
        stdout.execute(SetForegroundColor(Color::White))?;
        
        // Clear the status bar line
        for _ in 0..self.width {
            stdout.execute(Print(" "))?;
        }
        
        // Move back to the start of the line and print status info
        stdout.execute(cursor::MoveTo(0, self.height - 1))?;
        
        // Get current time
        let current_time = Local::now().format("%H:%M:%S").to_string();
        
        // Create status message with help info
        let status_msg = format!(" Ctrl+Q: Quit | Ctrl+B: Toggle Status Bar | {}", current_time);
        
        // Print the status message
        stdout.execute(Print(status_msg))?;
        
        // Print terminal dimensions on the right side
        let dims = format!("{}x{} ", self.width, self.height);
        let pos = self.width.saturating_sub(dims.len() as u16);
        stdout.execute(cursor::MoveTo(pos, self.height - 1))?;
        stdout.execute(Print(dims))?;
        
        // Reset colors
        stdout.execute(ResetColor)?;
        
        Ok(())
    }
    
    /// Render the current terminal state to stdout
    pub fn render(&self) -> Result<()> {
        let mut stdout = stdout();
        
        // Reset terminal state
        stdout.execute(Clear(ClearType::All))?;
        
        // Render the title bar
        self.render_title_bar()?;
        
        let screen = self.parser.screen();
        
        // Track current colors to minimize color changes
        let mut current_fg = None;
        let mut current_bg = None;
        
        // Render each row of the terminal (offset by 1 for the title bar)
        let content_height = if self.show_status_bar {
            self.height.saturating_sub(2)
        } else {
            self.height.saturating_sub(1)
        };
        
        for y in 0..content_height {
            if y >= screen.size().0 {
                break;
            }
            
            stdout.execute(cursor::MoveTo(0, y + 1))?;
            
            // Render each cell in the row
            for x in 0..self.width {
                if x >= screen.size().1 {
                    break;
                }
                
                // Fix: Properly handle Option<&Cell> by using if let
                if let Some(cell) = screen.cell(y, x) {
                    // Set foreground color if it changed
                    let cell_fg = map_vt100_color(cell.fgcolor());
                    if current_fg != Some(cell_fg) {
                        stdout.execute(SetForegroundColor(cell_fg))?;
                        current_fg = Some(cell_fg);
                    }
                    
                    // Set background color if it changed
                    let cell_bg = map_vt100_color(cell.bgcolor());
                    if current_bg != Some(cell_bg) {
                        stdout.execute(SetBackgroundColor(cell_bg))?;
                        current_bg = Some(cell_bg);
                    }
                    
                    // Print the cell content - Fix: use contents() instead of ch()
                    let text = cell.contents();
                    if text.is_empty() {
                        stdout.execute(Print(" "))?;
                    } else {
                        stdout.execute(Print(text))?;
                    }
                } else {
                    // Empty cell, just print a space
                    stdout.execute(Print(" "))?;
                }
            }
        }
        
        // Reset colors before rendering status bar
        stdout.execute(ResetColor)?;
        
        // Render the status bar
        self.render_status_bar()?;
        
        // Move cursor to the current cursor position in the terminal (offset by 1 for title bar)
        let (cursor_y, cursor_x) = screen.cursor_position();
        stdout.execute(cursor::MoveTo(cursor_x as u16, (cursor_y as u16) + 1))?;
        
        // Ensure all output is written
        stdout.flush()?;
        
        Ok(())
    }
}

/// Map vt100 color to crossterm Color
fn map_vt100_color(color: vt100::Color) -> Color {
    match color {
        vt100::Color::Default => Color::Reset,
        vt100::Color::Idx(0) => Color::Black,
        vt100::Color::Idx(1) => Color::DarkRed,
        vt100::Color::Idx(2) => Color::DarkGreen,
        vt100::Color::Idx(3) => Color::DarkYellow,
        vt100::Color::Idx(4) => Color::DarkBlue,
        vt100::Color::Idx(5) => Color::DarkMagenta,
        vt100::Color::Idx(6) => Color::DarkCyan,
        vt100::Color::Idx(7) => Color::Grey,
        vt100::Color::Idx(8) => Color::DarkGrey,
        vt100::Color::Idx(9) => Color::Red,
        vt100::Color::Idx(10) => Color::Green,
        vt100::Color::Idx(11) => Color::Yellow,
        vt100::Color::Idx(12) => Color::Blue,
        vt100::Color::Idx(13) => Color::Magenta,
        vt100::Color::Idx(14) => Color::Cyan,
        vt100::Color::Idx(15) => Color::White,
        vt100::Color::Idx(n) => {
            // Map 256-color palette
            if n < 232 {
                let r = (n - 16) / 36;
                let g = ((n - 16) % 36) / 6;
                let b = (n - 16) % 6;
                Color::Rgb { r: r as u8 * 42 + 36, g: g as u8 * 42 + 36, b: b as u8 * 42 + 36 }
            } else {
                // Grayscale colors
                let gray = (n - 232) * 10 + 8;
                Color::Rgb { r: gray as u8, g: gray as u8, b: gray as u8 }
            }
        },
        vt100::Color::Rgb(r, g, b) => Color::Rgb { r, g, b },
    }
}