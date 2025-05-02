use anyhow::anyhow;

#[derive(Clone, Debug)]
pub enum Command {
    Exit,
    Echo(String),
    Ls,
    LsDetailed,
    Pwd,
    Cd(String),
    Touch(String),
    Rm(String),
    Cat(String),
    Mkdir(String),
    MkdirP(String),
    Rmdir(String),
    RmdirR(String),
    Cp(String, String),
    CpR(String, String),
    Mv(String, String),
    Stat(String),
    Find(String, String),
    Grep(String, String),
    Ln(String, String),
}

impl TryFrom<&str> for Command {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let split_value: Vec<&str> = value.split_whitespace().collect();
        
        if split_value.is_empty() {
            return Err(anyhow!("Empty command"));
        }

        match split_value[0] {
            "exit" => Ok(Command::Exit),
            "ls" => {
                if split_value.len() > 1 && split_value[1] == "-l" {
                    Ok(Command::LsDetailed)
                } else {
                    Ok(Command::Ls)
                }
            },
            "echo" => {
                if split_value.len() < 2 {
                    Err(anyhow!("echo command requires an argument"))
                } else {
                    Ok(Command::Echo(split_value[1..].join(" ")))
                }
            }
            "pwd" => Ok(Command::Pwd),
            "cd" => {
                if split_value.len() < 2 {
                    Err(anyhow!("cd command requires an argument"))
                } else {
                    Ok(Command::Cd(split_value[1..].join(" ")))
                }
            }
            "touch" => {
                if split_value.len() < 2 {
                    Err(anyhow!("touch command requires an argument"))
                } else {
                    Ok(Command::Touch(split_value[1..].join(" ")))
                }
            }
            "rm" => {
                if split_value.len() < 2 {
                    Err(anyhow!("rm command requires an argument"))
                } else {
                    Ok(Command::Rm(split_value[1..].join(" ")))
                }
            }
            "cat" => {
                if split_value.len() < 2 {
                    Err(anyhow!("cat command requires an argument"))
                } else {
                    Ok(Command::Cat(split_value[1..].join(" ")))
                }
            }
            "mkdir" => {
                if split_value.len() < 2 {
                    Err(anyhow!("mkdir command requires an argument"))
                } else if split_value.len() > 2 && split_value[1] == "-p" {
                    Ok(Command::MkdirP(split_value[2..].join(" ")))
                } else {
                    Ok(Command::Mkdir(split_value[1..].join(" ")))
                }
            }
            "rmdir" => {
                if split_value.len() < 2 {
                    Err(anyhow!("rmdir command requires an argument"))
                } else if split_value.len() > 2 && split_value[1] == "-r" {
                    Ok(Command::RmdirR(split_value[2..].join(" ")))
                } else {
                    Ok(Command::Rmdir(split_value[1..].join(" ")))
                }
            }
            "cp" => {
                if split_value.len() < 3 {
                    Err(anyhow!("cp command requires source and destination arguments"))
                } else if split_value.len() > 3 && split_value[1] == "-r" {
                    Ok(Command::CpR(split_value[2].to_string(), split_value[3].to_string()))
                } else {
                    Ok(Command::Cp(split_value[1].to_string(), split_value[2].to_string()))
                }
            }
            "mv" => {
                if split_value.len() < 3 {
                    Err(anyhow!("mv command requires source and destination arguments"))
                } else {
                    Ok(Command::Mv(split_value[1].to_string(), split_value[2].to_string()))
                }
            }
            "stat" => {
                if split_value.len() < 2 {
                    Err(anyhow!("stat command requires a file path"))
                } else {
                    Ok(Command::Stat(split_value[1..].join(" ")))
                }
            }
            "find" => {
                if split_value.len() < 3 {
                    Err(anyhow!("find command requires directory and pattern arguments"))
                } else {
                    Ok(Command::Find(split_value[1].to_string(), split_value[2].to_string()))
                }
            }
            "grep" => {
                if split_value.len() < 3 {
                    Err(anyhow!("grep command requires file and pattern arguments"))
                } else {
                    Ok(Command::Grep(split_value[1].to_string(), split_value[2].to_string()))
                }
            }
            "ln" => {
                if split_value.len() < 3 {
                    Err(anyhow!("ln command requires target and link name arguments"))
                } else {
                    Ok(Command::Ln(split_value[1].to_string(), split_value[2].to_string()))
                }
            }
            _ => Err(anyhow!("Unknown command")),
        }
    }
}
