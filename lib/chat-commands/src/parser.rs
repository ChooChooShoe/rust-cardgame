
use std::str::FromStr;
use std::error;
use std::fmt;
use std::ops::Range;

#[derive(Debug, Hash, Eq, PartialEq)]
pub struct CommandLine {
    line: String,
    args: Vec<Range<usize>>,
}

const SINGLE: char = '\'';
const DOUBLE: char = '\"';
const SPACE: char = ' ';
const CMD_SLASH: char = '/';

impl CommandLine {
    pub fn from_line(input_line: &str) -> CommandLine {
        let mut args = Vec::new();
        let mut begin = 0;
        let mut end = 0;
        let mut state = State::Normal;
        //let mut is_last_quote_done = false;

        for (i, token) in input_line.char_indices() {
            match state {
                State::InSingleQuote => match token {
                    SINGLE => {
                        args.push(begin..i);
                        end = i + 1;
                        begin = end;
                        state = State::Normal;
                    }
                    _ => {
                        end = i + 1;
                    }
                },
                State::InDoubleQuote => match token {
                    DOUBLE => {
                        args.push(begin..i);
                        end = i + 1;
                        begin = end;
                        state = State::Normal;
                    }
                    _ => {
                        end = i + 1;
                    }
                },
                State::Normal => match token {
                    SINGLE => {
                        state = State::InSingleQuote;
                        begin = i + 1;
                    }
                    DOUBLE => {
                        state = State::InDoubleQuote;
                        begin = i + 1;
                    }
                    SPACE => {
                        if begin != end {
                            args.push(begin..end);
                        }
                        end = i + 1;
                        begin = end;
                    }
                    _ => {
                        if begin == 0 && token == CMD_SLASH {
                            begin = 1;
                            end = 1;
                        } else {
                            end = i + 1;
                        }
                    }
                },
            }
        }
        if state == State::Normal {
            if begin != end {
                args.push(begin..end);
            }
        }
        //assert_eq!(state, State::Normal);
        CommandLine { line: input_line.to_string(), args }
    }

    pub fn get(&self, i: usize) -> &str {
        &self.line[self.args[i].clone()]
    }

    pub fn get_num<F>(&self, i: usize) -> Result<F, &str> where F: FromStr {
        let arg = self.get(i);
        match arg.parse::<F>() {
            Ok(num) => Ok(num),
            Err(_) => Err(arg),
        }
    }
}

impl<'a> fmt::Display for CommandLine  {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut line = String::new();

        if self.args.len() == 0 {
            return write!(f, "")
        }

        let last = self.args.len() - 1;
        for i in 0..last {
            let arg = self.get(i);
            match arg.find(SPACE) {
                Some(_) => {
                    line.push(DOUBLE);
                    line.push_str(arg);
                    line.push(DOUBLE);
                    line.push(SPACE);
                } 
                None => {
                    line.push_str(arg);
                    line.push(SPACE);
                }
            }
        }
        line.push_str(self.get(last));

        write!(f, "{}", line)
    }
}

#[derive(Debug, Eq, PartialEq, PartialOrd, Ord)]
enum State {
    Normal,
    InSingleQuote,
    InDoubleQuote,
}

#[allow(unused)]
const EXAMPLE: &'static str = "

/help [page]
/help <command>
/msg <person> <message>...
/tp [target] <to>
/tp [target] <x> <y> [z]

";


pub struct Parser {

}
pub struct Arg {
    name: String,
    position: usize,
}

impl Arg {
    pub fn new(name: &str, position: usize) -> Arg {
        Arg {
            name: name.to_string(),
            position
        }
    }
    pub fn get_position(&self) -> usize{
        self.position
    }
}
//use std::collections::vec_map::VecMap;
use std::collections::HashMap;
//represents a single command that this app will understand.
pub struct AppCommand {
    aliases: Vec<String>,
    args: HashMap<usize, Arg>,
    about: String,
    version: String,
}

impl AppCommand {
    pub fn new(name: &str) -> AppCommand {
        AppCommand {
            aliases: vec!(name.to_string()),
            args: HashMap::new(),
            about: String::new(),
            version: String::from("0.1"),
        }
    }
    pub fn alias(mut self, alias: &str) -> AppCommand {
        self.aliases.push(alias.to_string());
        self
    }
    pub fn get_alias(&self) -> &str {
        &self.aliases.get(0).unwrap()
    }
    pub fn get_aliases(&self) -> &[String] {
        &self.aliases
    }
    pub fn about(mut self, about: &str) -> AppCommand {
        self.about = about.to_string();
        self
    }
    pub fn get_about(&self) -> &str {
        &self.about
    }
    
    pub fn version(mut self, version: &str) -> AppCommand {
        self.version = version.to_string();
        self
    }
    pub fn get_version(&self) -> &str {
        &self.version
    }
    
    pub fn arg(mut self, arg: Arg) -> AppCommand {
        self.args.insert(arg.get_position(), arg);
        self
    }
}