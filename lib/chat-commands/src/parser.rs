
use std::str::FromStr;
use std::error;
use std::fmt;
use std::ops::Range;

#[derive(Debug, Hash, Eq, PartialEq)]
pub struct CommandLine {
    pub line: String,
    pub args: Vec<Range<usize>>,
    invalid: bool
}

const SINGLE: char = '\'';
const DOUBLE: char = '\"';
const SPACE: char = ' ';
const CMD_SLASH: char = '/';

impl CommandLine {
    pub fn from_line(input_line: &str) -> CommandLine {
        let line = input_line.trim().to_string();
        let mut args = Vec::new();
        let mut begin = 0;
        let mut end = 0;
        let mut state = State::Normal;
        //let mut is_last_quote_done = false;

        for (i, token) in line.char_indices() {
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
        let invalid = State::Normal != state;
        CommandLine { line, args, invalid }
    }

    pub fn args(&self) -> Vec<&str> {
        let mut v = Vec::new();
        for range in &self.args {
            v.push(&self.line[range.clone()]);
        }
        v
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
    pub fn is_invalid(&self) -> bool {
        self.invalid
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
pub type CmdResult = result::Result<(), CmdError>;

#[derive(Debug, Clone,Eq,PartialEq)]
pub enum CmdError {
    InvalidLine(),
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
    EmptyCommand(),
}

impl fmt::Display for CmdError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &CmdError::InvalidLine() => write!(f, "Invalid command line. Are all quotes closed?"),
            &CmdError::NothingGiven() => write!(f, "No command given."),
            &CmdError::NotFound(ref x) => write!(f, "Unknown command: '{}'.", x),
            &CmdError::TooManyArgs{needs,got} => write!(f, "To many arguments given: needs {} got {}.", needs, got),
            &CmdError::NotEnoughArgs{needs,got} => write!(f, "Not Enough arguments given: needs {} got {}.", needs, got),
            &CmdError::InvalidArg{ref got,ref reason} => write!(f, "Invalid argument: '{}' was given when {}",got,reason),
            &CmdError::UnknownArg(ref x) => write!(f, "Unknown argument '{}'.", x),
            &CmdError::MissingArg(ref x) => write!(f, "Required argument '{}' is missing.", x),
            &CmdError::UnexpectedArg(ref x, ref y) => write!(f, "Unexpected argument '{}': try using '{}'.", x, y),
            &CmdError::Generic(ref x) => write!(f, "Command error: {}", x),
            &CmdError::NoPermission() => write!(f, "You do not have permission for this command"),
            _ => write!(f, "Unknown Error"),
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
            _ => "Unknown Command Error"
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
pub enum Type {
    Int,
    PositveInt,
    RealNum,
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
    pub fn typecheck(mut self, typec: Type) -> Arg {
        self.typec = typec;
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

#[derive(Debug)]
pub struct MatchedArgs<'a> {
    pub alias: &'a str,
    pub args_count: usize,
    pub args: &'a [&'a str],
}

fn empty_cmd(cmd: &AppCommand, args: MatchedArgs) -> CmdResult {
    Err(CmdError::EmptyCommand())
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
    pub callback: Box<Fn(&AppCommand,MatchedArgs) -> CmdResult + 'static>
}

impl AppCommand {
    pub fn new(name: &str) -> AppCommand {
        AppCommand {
            aliases: vec!(name.to_string()),
            args: HashMap::new(),
            about: String::new(),
            version: String::from("1.0"),
            callback: Box::new(empty_cmd)
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
    pub fn execute<F>(mut self, callback: F) -> AppCommand where F: Fn(&AppCommand,MatchedArgs) -> CmdResult + 'static  {
        self.callback = Box::new(callback);
        self
    }

    pub fn get_examples(&self) -> Vec<&str> {
        let mut v = vec!();
        v.push("");
        v
    }
}