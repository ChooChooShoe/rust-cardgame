#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
    #[test]
    fn stringify_1() {
        assert_eq!(
            format!("{}", CommandLine::from_line("/help me do this")),
            format!("{}", "help me do this")
        );
    }
    #[test]
    fn stringify_2() {
        assert_eq!(
            format!("{}", CommandLine::from_line("")),
            format!("{}", "")
        );
    }
    #[test]
    fn stringify_3() {
        assert_eq!(
            format!("{}", CommandLine::from_line("/one")),
            format!("{}", "one")
        );
    }
}

#[macro_use]
extern crate log;

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


pub type CmdResult = std::result::Result<String, CmdError>;

#[derive(Debug, Clone,Eq,PartialEq)]
pub enum CmdError {
    NothingGiven(),
    NotFound(String),
    TooManyArgs{needs: usize, got: usize},
    NotEnoughArgs{needs: usize, got: usize},
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
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        None
    }
}
#[allow(dead_code)]
const EXAMPLE: &'static str = "

Usage:

/help [page: int]
/help <command name>
/play <card> [pos]
/msg <to> <message>

Alises:
/help   /?  /lookup
";

//old
#[allow(unused_macros)]
macro_rules! create_run_function {
    ($run_name:ident, $do_name:ident, $req:expr, $opt:expr) => {

        fn $run_name(&self, alias: &str, arg_count: usize, args: &[&str]) -> CmdResult {
            debug!("Command {} run.", stringify!($func_name));
            match arg_count {
                0 => self.$do_name(args[0]),
                1 => self.$do_name(args[0], args[1]),
                2 => self.$do_name(args[0], args[1], args[2]),
                _ => Err(CmdError::TooManyArgs{needs: 1, got: arg_count}),
            }
        }
    };
}

macro_rules! make_command_centre {
    ( $( $command_alias:tt $(, $echo_alias:tt)*  => $run_func:ident ),* ) => {
        pub trait CommandCentre {
            $(
                fn $run_func(&self, alias: &str, arg_count: usize, args: &[&str]) -> CmdResult;
            )*

            fn run(&self, args:  &[&str]) -> CmdResult {
                if args.len() == 0 {
                    Err(CmdError::NothingGiven())
                } else {
                    let args_count = args.len() - 1;
                    match args[0] {
                        $(
                            $command_alias => self.$run_func(args[0], args_count, &args[1..]),
                            $(
                                $echo_alias => self.$run_func(args[0], args_count, &args[1..]),
                            )*
                        )*
                        _ => Err(CmdError::NotFound(args[0].to_string())),
                    }
                }
            }
        }
    };
}

make_command_centre!(
    "help","?" => run_help,
    "play" => run_play,
    "msg" => run_msg
);
struct CmdImp;

impl CmdImp {
    // Gets general help
    fn do_help(&self, alias: &str) -> CmdResult {Err(CmdError::NoPermission())}
    // Gets help for given command.
    fn do_help_command(&self, alias: &str, command: &str) -> CmdResult {Err(CmdError::NoPermission())}
    // Gets help at page #
    fn do_help_page(&self, alias: &str, page: usize) -> CmdResult {Err(CmdError::NoPermission())}

    // Gets general help
    fn do_msg(&self, alias: &str, to: &str, message: &str) -> CmdResult {Err(CmdError::NoPermission())}
}

impl CommandCentre for CmdImp {
    
    fn run_help(&self, alias: &str, arg_count: usize, args: &[&str]) -> CmdResult {
        trace!("Command 'help' run.");
        match arg_count {
            0 => self.do_help(args[0]),
            1 => {
                match args[1].parse::<usize>() {
                    Ok(page) => self.do_help_page(args[0], page),
                    Err(_) => self.do_help_command(args[0], args[1]),
                }
            },
            _ => Err(CmdError::TooManyArgs{needs: 1, got: arg_count}),
        }

    }
    fn run_play(&self, alias: &str, arg_count: usize, args: &[&str]) -> CmdResult {
        trace!("Command 'play' run.");
        Ok("play".to_string())
    }
    fn run_msg(&self, alias: &str, arg_count: usize, args: &[&str]) -> CmdResult {
        trace!("Command 'msg' run.");

        match args[0] {
            "ok" => {},
            "5fff" | "dgg" => {},
            _ => { }
        }

        match arg_count {
            0|1 => Err(CmdError::NotEnoughArgs{needs: 2, got: 1}),
            2 => {
                self.do_msg(args[0], args[1], args[2])
            },
            _ => Err(CmdError::TooManyArgs{needs: 1, got: arg_count}),
        }
    }    
}