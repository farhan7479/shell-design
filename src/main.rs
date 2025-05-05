use command::Command;
use errors::CrateResult;
use colored::*;
use crossterm::terminal::size;
use std::env;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt},
    task::JoinHandle,
};
use std::process::Command as ProcessCommand;

mod command;
mod errors;
mod helpers;
mod terminal;

fn spawn_user_input_handler() -> JoinHandle<CrateResult<()>> {
    tokio::spawn(async {
        let stdin = tokio::io::stdin();
        let stdout = tokio::io::stdout();

        let mut reader = tokio::io::BufReader::new(stdin).lines();
        let mut stdout = tokio::io::BufWriter::new(stdout);

        // Display a colorful welcome message with ASCII art
        let welcome_message = format!(
            r#"
{}
  _____ _          _ _   ____            _              
 / ____| |        | | | |  _ \          (_)             
| (___ | |__   ___| | | | |_) | __ _ ___ _  ___ ___ ___ 
 \___ \| '_ \ / _ \ | | |  _ < / _` / __| |/ __/ __/ __|
 ____) | | | |  __/ | | | |_) | (_| \__ \ | (__\__ \__ \
|_____/|_| |_|\___|_|_| |____/ \__,_|___/_|\___|___/___/
                              by Farhan Shahid                                
Welcome to the Shell Basics v1.0! Type '{}' to see available commands.
{}"#,
            "━".repeat(60).bright_blue(),
            "help".bright_yellow(),
            "━".repeat(60).bright_blue()
        );

        stdout.write(welcome_message.as_bytes()).await?;
        stdout.write(b"\n").await?;

        loop {
            // Generate beautiful prompt with username and current directory
            let prompt = generate_prompt()?;
            stdout.write(prompt.as_bytes()).await?;
            stdout.flush().await?;

            if let Ok(Some(line)) = reader.next_line().await {
                let trimmed_line = line.trim();
                
                if trimmed_line.is_empty() {
                    continue;
                }
                
                if trimmed_line == "help" {
                    print_help();
                    continue;
                }
                
                let command = handle_new_line(&trimmed_line).await;

                if let Ok(command) = &command {
                    match command {
                        Command::Exit => {
                            println!("{}", "Exiting the shell. Goodbye!".bright_cyan());
                            break;
                        }
                        _ => {}
                    }
                } else {
                    eprintln!("{} {}", "Error:".bright_red(), command.err().unwrap());
                }
            }
        }

        Ok(())
    })
}

fn get_git_branch() -> Option<String> {
    let output = ProcessCommand::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .ok()?;
    
    if output.status.success() {
        String::from_utf8(output.stdout)
            .ok()
            .map(|s| s.trim().to_string())
    } else {
        None
    }
}

fn is_git_repository() -> bool {
    ProcessCommand::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn generate_prompt() -> CrateResult<String> {
    // Get username - fallback to "user" if we can't get it
    let username = std::env::var("USER").unwrap_or_else(|_| "farhan".to_string());
    
    // Get current directory
    let current_dir = std::env::current_dir()?;
    let dir_name = current_dir
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| "~".to_string());
    
    // Get parent directory
    let parent_dir = current_dir
        .parent()
        .and_then(|p| p.file_name())
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| "~".to_string());
    
    // Get git branch if in a git repository
    let git_branch_info = if is_git_repository() {
        if let Some(branch) = get_git_branch() {
            format!(" on {}", branch.purple().bold())
        } else {
            String::new()
        }
    } else {
        String::new()
    };
    
    // Format the prompt with colors
    let prompt = format!(
        "{} {} {} {} {} {}{} ", 
        "┌─[".bright_green(),
        username.bright_cyan(),
        "@shell".bright_blue(),
        "]─[".bright_green(),
        format!("{}/{}", parent_dir, dir_name).yellow(),
        "]".bright_green(),
        git_branch_info
    );
    
    // Add a new line and the input prompt
    let prompt = format!(
        "{}\n{}",
        prompt,
        "└─$ ".bright_green()
    );
    
    Ok(prompt)
}

fn print_help() {
    println!("\n{}", "=== Available Commands ===".bright_yellow().bold());
    
    println!("{}", "File Operations:".cyan().bold());
    println!("  {} - {}", "ls".green(), "List files in the current directory");
    println!("  {} - {}", "ls -l".green(), "List files with detailed information");
    println!("  {} - {}", "pwd".green(), "Print working directory");
    println!("  {} - {}", "cd <directory>".green(), "Change directory");
    println!("  {} - {}", "touch <file>".green(), "Create a new file or update timestamp");
    println!("  {} - {}", "rm <file>".green(), "Remove a file");
    println!("  {} - {}", "cat <file>".green(), "Display file contents");
    
    println!("\n{}", "Directory Operations:".cyan().bold());
    println!("  {} - {}", "mkdir <directory>".green(), "Create a directory");
    println!("  {} - {}", "mkdir -p <directory>".green(), "Create a directory and its parents");
    println!("  {} - {}", "rmdir <directory>".green(), "Remove an empty directory");
    println!("  {} - {}", "rmdir -r <directory>".green(), "Remove a directory and its contents");
    
    println!("\n{}", "File Management:".cyan().bold());
    println!("  {} - {}", "cp <source> <dest>".green(), "Copy a file");
    println!("  {} - {}", "cp -r <source>".green(), "Copy directory recursively");
    println!("  {} - {}", "mv <source> <dest>".green(), "Move/rename files or directories");
    println!("  {} - {}", "stat <file/dir>".green(), "Display file or directory information");
    println!("  {} - {}", "ln <target> <link_name>".green(), "Create symbolic link");
    
    println!("\n{}", "Search and Information:".cyan().bold());
    println!("  {} - {}", "find <dir> <pattern>".green(), "Find files matching pattern");
    println!("  {} - {}", "grep <file> <pattern>".green(), "Search for pattern in file");
    println!("  {} - {}", "echo <text>".green(), "Print text to the terminal");
    
    println!("\n{}", "Shell Control:".cyan().bold());
    println!("  {} - {}", "help".green(), "Display this help message");
    println!("  {} - {}", "exit".green(), "Exit the shell");
    
    println!("");
}

async fn handle_new_line(line: &str) -> CrateResult<Command> {
    let command: Command = line.try_into()?;

    match command.clone() {
        Command::Ls => {
            helpers::ls()?;
        }
        Command::LsDetailed => {
            helpers::ls_detailed()?;
        }
        Command::Echo(s) => {
            println!("{}", s);
        }
        Command::Pwd => {
            println!("{}", helpers::pwd()?.bright_yellow());
        }
        Command::Cd(s) => {
            helpers::cd(&s)?;
        }
        Command::Touch(s) => {
            helpers::touch(&s)?;
            println!("{} {}", "Created/Updated:".bright_green(), s);
        }
        Command::Rm(s) => {
            helpers::rm(&s)?;
            println!("{} {}", "Removed:".bright_red(), s);
        }
        Command::Cat(s) => {
            let contents = helpers::cat(&s)?;
            println!("{}\n{}\n{}", 
                format!("=== {} ===", s).bright_yellow(), 
                contents,
                "==========".bright_yellow());
        }
        Command::Mkdir(s) => {
            helpers::mkdir(&s)?;
            println!("{} {}", "Directory created:".bright_green(), s);
        }
        Command::MkdirP(s) => {
            helpers::mkdir_p(&s)?;
            println!("{} {}", "Directory hierarchy created:".bright_green(), s);
        }
        Command::Rmdir(s) => {
            helpers::rmdir(&s)?;
            println!("{} {}", "Directory removed:".bright_red(), s);
        }
        Command::RmdirR(s) => {
            helpers::rmdir_r(&s)?;
            println!("{} {}", "Directory and contents removed:".bright_red(), s);
        }
        Command::Cp(src, dest) => {
            helpers::cp(&src, &dest)?;
            println!("{} '{}' → '{}'", "Copied:".bright_green(), src, dest);
        }
        Command::CpR(src, dest) => {
            helpers::cp_r(&src, &dest)?;
            println!("{} '{}' → '{}'", "Recursively copied:".bright_green(), src, dest);
        }
        Command::Mv(src, dest) => {
            helpers::mv(&src, &dest)?;
            println!("{} '{}' → '{}'", "Moved:".bright_blue(), src, dest);
        }
        Command::Stat(path) => {
            let info = helpers::stat(&path)?;
            println!("{}\n{}", format!("=== Statistics for {} ===", path).bright_yellow(), info);
        }
        Command::Find(dir, pattern) => {
            let results = helpers::find(&dir, &pattern)?;
            println!("{} {} {}", 
                "Found".bright_green(), 
                results.len().to_string().yellow(), 
                "matches:".bright_green());
            
            for path in results {
                println!("  {}", path.display().to_string().cyan());
            }
        }
        Command::Grep(file, pattern) => {
            let results = helpers::grep(&file, &pattern)?;
            if results.is_empty() {
                println!("{} {}", "No matches found in".yellow(), file);
            } else {
                println!("{} {}:", "Matches in".bright_green(), file.yellow());
                
                // Colorize the output: line numbers in yellow, matched text highlighted
                for line in results.lines() {
                    if let Some(pos) = line.find(':') {
                        let (line_num, content) = line.split_at(pos + 1);
                        println!("{}{}", line_num.yellow(), content);
                    } else {
                        println!("{}", line);
                    }
                }
            }
        }
        Command::Ln(target, link_name) => {
            helpers::ln(&target, &link_name)?;
            println!("{} '{}' → '{}'", "Created symbolic link:".bright_green(), link_name, target);
        }
        _ => {}
    }
    Ok(command)
}

#[tokio::main]
async fn main() {
    // Enable colored output
    colored::control::set_override(true);
    
    // Check command-line arguments
    let args: Vec<String> = env::args().collect();
    
    if args.len() > 1 && args[1] == "--shell-mode" {
        // Running in shell mode (spawned by the terminal emulator)
        // Start the user input handler directly
        let user_input_handler = spawn_user_input_handler().await;

        if let Ok(Err(e)) = user_input_handler {
            eprintln!("{} {}", "Shell Error:".bright_red().bold(), e);
        }
    } else {
        // Running as a terminal emulator
        match run_terminal_emulator() {
            Ok(_) => (),
            Err(e) => eprintln!("{} {}", "Terminal Error:".bright_red().bold(), e),
        }
    }
}

/// Run the program as a terminal emulator
fn run_terminal_emulator() -> CrateResult<()> {
    // Get terminal size
    let (width, height) = size()?;
    
    // Create and run the terminal emulator
    let mut term = terminal::Terminal::new(width, height)?;
    term.run()?;
    
    Ok(())
}
