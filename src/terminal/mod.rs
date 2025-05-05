use anyhow::Result;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::stdout;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub mod input;
pub mod pty;
pub mod renderer;

use input::{InputEvent, InputHandler};
use pty::TerminalPty;
use renderer::TerminalRenderer;

/// Terminal emulator that combines PTY, renderer, and input handling
pub struct Terminal {
    pty: Arc<TerminalPty>,
    renderer: Arc<Mutex<TerminalRenderer>>,
    input_handler: InputHandler,
    width: u16,
    height: u16,
    running: bool,
    current_dir: String,
}

impl Terminal {
    /// Create a new terminal emulator
    pub fn new(width: u16, height: u16) -> Result<Self> {
        // Create a PTY that runs the current binary as our shell
        // We'll spawn our own shell process in this PTY
        let path = std::env::current_exe()?;
        let path_str = path.to_string_lossy();
        
        // Pass a special flag to indicate we're running in shell mode
        // This helps avoid recursion (terminal spawning terminal)
        let pty = TerminalPty::new(&path_str, &["--shell-mode"])?;
        
        // Create the renderer with terminal dimensions
        let renderer = TerminalRenderer::new(width, height);
        
        // Create input handler
        let input_handler = InputHandler::new();
        
        // Get current directory for title
        let current_dir = std::env::current_dir()?
            .to_string_lossy()
            .to_string();
        
        Ok(Self {
            pty: Arc::new(pty),
            renderer: Arc::new(Mutex::new(renderer)),
            input_handler,
            width,
            height,
            running: false,
            current_dir,
        })
    }
    
    /// Update the terminal title with current directory
    fn update_title(&mut self) -> Result<()> {
        // Get a short version of the current directory for display
        let home_dir = dirs_next::home_dir().map(|p| p.to_string_lossy().to_string());
        
        // Convert home directory to "~" for cleaner display
        let title = if let Some(home) = &home_dir {
            if self.current_dir.starts_with(home) {
                let path = self.current_dir.replacen(home, "~", 1);
                format!("Terminal - {}", path)
            } else {
                format!("Terminal - {}", self.current_dir)
            }
        } else {
            format!("Terminal - {}", self.current_dir)
        };
        
        // Update the renderer title
        self.renderer.lock().unwrap().set_title(title);
        
        Ok(())
    }
    
    /// Start the terminal emulator
    pub fn run(&mut self) -> Result<()> {
        // Setup terminal
        enable_raw_mode()?;
        execute!(stdout(), EnterAlternateScreen)?;
        
        // Update title with current directory
        self.update_title()?;
        
        // Spawn a thread to read from the PTY and update the renderer
        self.setup_pty_reader()?;
        
        self.running = true;
        
        // Main event loop
        while self.running {
            // Check for input events
            match self.input_handler.poll_event(100)? {
                InputEvent::Key(key) => {
                    // Check for special key combinations first
                    if key.code == crossterm::event::KeyCode::Char('q') 
                        && key.modifiers == crossterm::event::KeyModifiers::CONTROL {
                        // Ctrl+Q: Exit the terminal
                        self.running = false;
                        continue;
                    } else if key.code == crossterm::event::KeyCode::Char('b')
                        && key.modifiers == crossterm::event::KeyModifiers::CONTROL {
                        // Ctrl+B: Toggle status bar
                        self.renderer.lock().unwrap().toggle_status_bar();
                        continue;
                    } else if key.code == crossterm::event::KeyCode::Char('r')
                        && key.modifiers == crossterm::event::KeyModifiers::CONTROL {
                        // Ctrl+R: Toggle raw mode for debugging
                        self.renderer.lock().unwrap().toggle_raw_mode();
                        continue;
                    }
                    
                    // Process regular keyboard input
                    let input_bytes = self.input_handler.process_key_input(key);
                    if !input_bytes.is_empty() {
                        self.pty.write(&input_bytes)?;
                    }
                },
                InputEvent::Resize(width, height) => {
                    // Handle terminal resize
                    self.width = width;
                    self.height = height;
                    
                    // Resize both PTY and renderer
                    self.pty.resize(height, width)?;
                    self.renderer.lock().unwrap().resize(width, height);
                },
                InputEvent::None => {
                    // No input event, sleep briefly to avoid CPU spinning
                    thread::sleep(Duration::from_millis(10));
                    
                    // Periodically check for directory changes (every 1 second)
                    static mut COUNTER: u64 = 0;
                    unsafe {
                        COUNTER += 1;
                        if COUNTER % 100 == 0 { // 100 * 10ms = 1 second
                            // Check for directory changes to update title
                            if let Ok(dir) = std::env::current_dir() {
                                let dir_str = dir.to_string_lossy().to_string();
                                if dir_str != self.current_dir {
                                    self.current_dir = dir_str;
                                    self.update_title()?;
                                }
                            }
                        }
                    }
                },
            }
            
            // Render current terminal state
            self.renderer.lock().unwrap().render()?;
        }
        
        // Cleanup terminal
        disable_raw_mode()?;
        execute!(stdout(), LeaveAlternateScreen)?;
        
        Ok(())
    }
    
    /// Setup a thread to read from the PTY and update the renderer
    fn setup_pty_reader(&self) -> Result<()> {
        let renderer = Arc::clone(&self.renderer);
        
        self.pty.spawn_reader(move |data| {
            // Update the renderer with the new data from PTY
            let mut renderer = renderer.lock().unwrap();
            renderer.process_output(data);
        })?;
        
        Ok(())
    }
}