use anyhow::Result;
use portable_pty::{native_pty_system, CommandBuilder, PtyPair, PtySize, PtySystem};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;

/// Manages a pseudo-terminal for running the shell
pub struct TerminalPty {
    reader: Arc<Mutex<Box<dyn Read + Send>>>,
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
    pair: Option<PtyPair>,
}

impl TerminalPty {
    /// Creates a new PTY running the specified command
    pub fn new(shell_command: &str, args: &[&str]) -> Result<Self> {
        // Create a new native PTY system for the current platform
        let pty_system = native_pty_system();
        
        // Create PTY with initial size (80x24 is standard)
        let pair = pty_system.openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })?;
        
        // Create the command to run in the PTY
        let mut cmd = CommandBuilder::new(shell_command);
        for arg in args {
            cmd.arg(arg);
        }
        
        // Set TERM environment variable for proper terminal capabilities
        cmd.env("TERM", "xterm-256color");
        
        // Spawn the command in the PTY
        let _child = pair.slave.spawn_command(cmd)?;
        
        // Get reader and writer for the master side of the PTY
        let reader = pair.master.try_clone_reader()?;
        // Fix: portable-pty API doesn't have try_clone_writer, use write_output method
        let writer = Box::new(pair.master.take_writer()?);
        
        Ok(Self {
            reader: Arc::new(Mutex::new(reader)),
            writer: Arc::new(Mutex::new(writer)),
            pair: Some(pair),
        })
    }
    
    /// Read data from the PTY
    pub fn read(&self, buf: &mut [u8]) -> Result<usize> {
        let mut reader = self.reader.lock().unwrap();
        let n = reader.read(buf)?;
        Ok(n)
    }
    
    /// Write data to the PTY
    pub fn write(&self, buf: &[u8]) -> Result<usize> {
        let mut writer = self.writer.lock().unwrap();
        writer.write_all(buf)?;
        writer.flush()?;
        Ok(buf.len())
    }
    
    /// Resize the PTY
    pub fn resize(&self, rows: u16, cols: u16) -> Result<()> {
        if let Some(ref pair) = self.pair {
            pair.master.resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })?;
        }
        Ok(())
    }
    
    /// Spawn a background reader thread that calls the provided callback
    /// when data is available from the PTY
    pub fn spawn_reader<F>(&self, callback: F) -> Result<()>
    where
        F: Fn(&[u8]) + Send + 'static,
    {
        let reader = Arc::clone(&self.reader);
        
        thread::spawn(move || {
            let mut buffer = [0u8; 4096];
            loop {
                let n = {
                    let mut reader = reader.lock().unwrap();
                    match reader.read(&mut buffer) {
                        Ok(0) => break, // EOF
                        Ok(n) => n,
                        Err(_) => break, // Error reading
                    }
                };
                
                callback(&buffer[..n]);
            }
        });
        
        Ok(())
    }
}