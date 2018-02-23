
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

use std::result;
pub type CmdResult = result::Result<String, CmdError>;

#[derive(Debug, Clone,Eq,PartialEq)]
pub enum CmdError {
    NothingGiven(),
    NotFound(String),
    TooManyArgs{needs: usize, got: usize},
    NotEnoughArgs{needs: usize, got: usize},
    InvalidArg{got: String, reason: String},
    UnknownArg(String),
    MissingArg(String),
    UnexpectedArg(String,String),
    Generic(String),
    NoPermission(),
}

impl fmt::Display for CmdError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &CmdError::NothingGiven() => write!(f, "No command given."),
            &CmdError::NotFound(ref x) => write!(f, "Unknown command: {}.", x),
            &CmdError::TooManyArgs{needs,got} => write!(f, "To many arguments given: needs {} got {}.", needs, got),
            &CmdError::NotEnoughArgs{needs,got} => write!(f, "Not Enough arguments given: needs {} got {}.", needs, got),
            &CmdError::InvalidArg{ref got,ref reason} => write!(f, "Invalid argument: '{}' was given when {}",got,reason),
            &CmdError::UnknownArg(ref x) => write!(f, "Unknown argument '{}'.", x),
            &CmdError::MissingArg(ref x) => write!(f, "Required argument '{}' is missing.", x),
            &CmdError::UnexpectedArg(ref x, ref y) => write!(f, "Unexpected argument '{}': try using '{}'.", x, y),
            &CmdError::Generic(ref x) => write!(f, "Command error: {}", x),
            &CmdError::NoPermission() => write!(f, "You do not have permission for this command"),
        }
    }
}

// This is important for other errors to wrap this one.
impl error::Error for CmdError {
    fn description(&self) -> &str {
        match self {
            &CmdError::NothingGiven() => "No command given.",
            &CmdError::NotFound(_) => "Unknown command.",
            &CmdError::TooManyArgs{needs,got} => "To many arguments given.",
            &CmdError::NotEnoughArgs{needs,got} => "Not Enough arguments given.",
            &CmdError::UnknownArg(_) => "Unknown argument.",
            &CmdError::MissingArg(_) => "Required argument.",
            &CmdError::UnexpectedArg(_,_) => "Unexpected argument.",
            &CmdError::Generic(_) => "Command error.",
            &CmdError::NoPermission() => "You do not have permission for this command",
            _ => "CmdError"
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        None
    }
}

#[derive(Debug)]
pub struct Parser {


}
#[derive(Debug,Eq,PartialEq)]
pub enum Type{
    U32,
    I32,
    F64,
    Bool,
    Str,
}
use std::error::Error;

impl Type {

}

#[derive(Debug)]
pub struct Arg {
    name: String,
    position: usize,
    typec: Type,
    required: bool,
    value: String,
}

impl fmt::Display for Arg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.required {
            write!(f, "<{}>", self.name)
        } else {
            write!(f, "[{}]", self.name)
        }
    }
}

impl Arg {
    pub fn new(name: &str, position: usize) -> Arg {
        Arg {
            name: name.to_string(),
            position,
            typec: Type::Str,
            required: false,
            value: String::new(),
        }
    }
    pub fn required(mut self, required: bool) -> Arg {
        self.required = required;
        self
    }

    pub fn get_position(&self) -> usize {
        self.position
    }
    pub fn get_value(&self) -> &str {
        &self.value
    }
    pub fn parse_value<T: FromStr>(&self) -> Result<T,CmdError> {
        match self.value.parse::<T>() {
            Ok(x) => Ok(x),
            Err(_) => {
                Err(CmdError::InvalidArg{got: self.value.to_string(), reason: "number was required".to_string()})
            }
        }
    }
}
//use std::collections::vec_map::VecMap;
use std::collections::HashMap;
//represents a single command that this app will understand.
//#[derive(Debug)]
pub struct AppCommand {
    aliases: Vec<String>,
    args: HashMap<usize, Arg>,
    about: String,
    version: String,
    func: Option<Box<FnOnce(AppCommand) -> CmdResult>>
}

impl AppCommand {
    pub fn new(name: &str) -> AppCommand {
        AppCommand {
            aliases: vec!(name.to_string()),
            args: HashMap::new(),
            about: String::new(),
            version: String::from("0.1"),
            func: None
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
    pub fn execute<F: 'static>(mut self, func: F) -> AppCommand where F: FnOnce(AppCommand) -> CmdResult {
        self.func = Some(Box::new(func));
        self
    }
}