use std::str::FromStr;

#[derive(Debug)]
pub enum Command {
    Ping,
    Pong,
}

impl FromStr for Command {
    type Err = ();

    fn from_str(input: &str) -> Result<Command, Self::Err> {
        match input {
            "ping" => Ok(Command::Ping),
            "pong" => Ok(Command::Pong),
            _ => Err(()),
        }
    }
}

impl Command {
    pub fn run(&self) -> String {
        match self {
            Command::Ping => String::from("Pong!"),
            Command::Pong => String::from("Ping!"),
        }
    }
}
