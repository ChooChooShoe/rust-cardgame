#[cfg(test)]
mod tests {
    use super::*;

    pub fn command_from_vec(vec: Vec<&str>) -> Result<CommandLine,()> {
        let mut args = Vec::with_capacity(vec.len());
        for s in vec {
            args.push(s.to_string());
        }
        Ok(CommandLine { args })
    }
    #[derive(Debug,Eq,PartialEq)]
    struct ex;
    impl CommandCentre for ex{

        // Gets general help
        fn do_ok(&self, alias: &str) -> Result {Err(CmdError::NoPermission())}
        fn do_help(&self, alias: &str) -> Result {Err(CmdError::NoPermission())}
        // Gets help for given command.
        fn do_help_command(&self, alias: &str, command: &str) -> Result {Err(CmdError::NoPermission())}
        // Gets help at page #
        fn do_help_page(&self, alias: &str, page: usize) -> Result {Err(CmdError::NoPermission())}

        // Gets general help
        fn do_msg(&self, alias: &str, to: &str, message: &str) -> Result {Err(CmdError::NoPermission())}
    }

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
    #[test]
    fn stringify_1() {
        assert_eq!(
            format!("{}", CommandLine::from_line("/help me do this").unwrap()),
            format!("{}", "help me do this")
        );
    }
    #[test]
    fn stringify_2() {
        assert_eq!(
            format!("{}", CommandLine::from_line("").unwrap()),
            format!("{}", "")
        );
    }
    #[test]
    fn stringify_3() {
        assert_eq!(
            format!("{}", CommandLine::from_line("/one").unwrap()),
            format!("{}", "one")
        );
    }

    #[test]
    fn simple_help() {
        assert_eq!(
            CommandLine::from_line("/help me"),
            command_from_vec(vec!["help", "me"])
        );
    }
    #[test]
    fn simple_help_slash() {
        assert_eq!(
            CommandLine::from_line("//help"),
            command_from_vec(vec!["/help"])
        );
    }
    #[test]
    fn simple_help_2() {
        assert_eq!(
            CommandLine::from_line(" help "),
            command_from_vec(vec!["help"])
        );
    }
    #[test]
    fn simple_empty1() {
        assert_eq!(CommandLine::from_line(""), command_from_vec(Vec::new()));
    }
    #[test]
    fn simple_empty2() {
        assert_eq!(
            CommandLine::from_line(" "),
            command_from_vec(Vec::new())
        );
    }
    #[test]
    fn simple_empty3() {
        assert_eq!(
            CommandLine::from_line("/ "),
            command_from_vec(Vec::new())
        );
    }
    #[test]
    fn simple_empty4() {
        assert_eq!(
            CommandLine::from_line("/ ok"),
            command_from_vec(vec!["ok"])
        );
    }
    #[test]
    fn quote() {
        assert_eq!(
            CommandLine::from_line("open 'the doors'"),
            command_from_vec(vec!["open", "the doors"])
        );
    }
    #[test]
    fn quote2() {
        assert_eq!(
            CommandLine::from_line("/open ''  "),
            command_from_vec(vec!["open", ""])
        );
    }
    #[test]
    fn quote3() {
        assert_eq!(
            CommandLine::from_line("/see \"it is\""),
            command_from_vec(vec!["see", "it is"])
        );
    }
    #[test]
    fn quote_lvl_1() {
        assert_eq!(
            CommandLine::from_line("see \"it's\""),
            command_from_vec(vec!["see", "it's"])
        );
    }
    #[test]
    fn big_1() {
        assert_eq!(
            CommandLine::from_line("open file \"c:\\folder\\file.txt\" \"to dir\""),
            command_from_vec(vec!["open", "file", "c:\\folder\\file.txt", "to dir"])
        );
    }
    #[test]
    fn big_2() {
        assert_eq!(
            CommandLine::from_line("open file \"c:\\folder\\file.txt\" dir"),
            command_from_vec(vec!["open", "file", "c:\\folder\\file.txt", "dir"])
        );
    }
    #[test]
    fn big_3() {
        assert_eq!(
            CommandLine::from_line("open file \"quoate\"noq"),
            command_from_vec(vec!["open", "file", "quoate", "noq"])
        );
    }
    #[test]
    fn big_4() {
        assert_eq!(
            CommandLine::from_line("open \"q1\"'q2'"),
            command_from_vec(vec!["open", "q1", "q2"])
        );
    }
    #[test]
    fn err_1() {
        assert_eq!(
            CommandLine::from_line("open \"q1'ww\"q2"),
            command_from_vec(vec!["open", "q1'ww", "q2"])
        );
    }
    #[test]
    fn err_q1() {
        assert_eq!(
            CommandLine::from_line("open thi's"),
            command_from_vec(vec!["open"])
        );
    }
    #[test]
    fn err_q2() {
        assert_eq!(
            CommandLine::from_line("open thi\"s"),
            command_from_vec(vec!["open"])
        );
    }
    #[test]
    fn test_macro() {
        let ex = ex {};
        assert_eq!(ex.run(vec!("ok")), Err(CmdError::NoPermission()));
    }
}

#[macro_use]
extern crate log;

use std::error;
use std::fmt;


#[derive(Debug, Hash, Eq, PartialEq,Clone)]
pub struct CommandLine {
    args: Vec<String>,
}
const SINGLE: char = '\'';
const DOUBLE: char = '\"';
const SPACE: char = ' ';
const CMD_SLASH: char = '/';

impl CommandLine {
    pub fn from_line(input_line: &str) -> Result<CommandLine,()> {
        let line = input_line.trim();
        let mut args = Vec::new();
        let mut begin = 0;
        let mut end = 0;
        let mut state = State::Normal;
        //let mut is_last_quote_done = false;

        for (i, token) in line.char_indices() {
            match state {
                State::InSingleQuote => match token {
                    SINGLE => {
                        args.push(line[begin..i].to_string());
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
                        args.push(line[begin..i].to_string());
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
                            args.push(line[begin..end].to_string());
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
                args.push(line[begin..end].to_string());
            }
        }
        //assert_eq!(state, State::Normal);
        Ok(CommandLine { args })
    }
}

use std::fmt;
impl fmt::Display for CommandLine  {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut line = String::new();

        if self.args.len() == 0 {
            return write!(f, "")
        }

        let last = self.args.len() - 1;
        for arg in &self.args[0..last] {
            line.push_str(&arg);
            line.push_str(" ");
        }
            line.push_str(&self.args[last]);

        write!(f, "{}", line)
    }
}

#[derive(Debug, Eq, PartialEq, PartialOrd, Ord)]
enum State {
    Normal,
    InSingleQuote,
    InDoubleQuote,
}


pub type Result = std::result::Result<String, CmdError>;

#[derive(Debug, Clone,Eq,PartialEq)]
pub enum CmdError {
    NothingGiven(),
    NotFound(String),
    TooManyArgs(usize,usize),
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
            &CmdError::TooManyArgs(ref x, ref y) => write!(f, "To many arguments given: needs {} got {}.", x, y),
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
        "error"
    }

    fn cause(&self) -> Option<&error::Error> {
        None
    }
}
#[allow(dead_code)]
const EXAMPLE: &'static str = "

Usage:

/help [command|page]
/play <card> [pos]
/msg <to> <message>

Alises:
/help   /?  /lookup
";

macro_rules! create_run_function {
    ($run_name:ident, $do_name:ident, $req:expr, $opt:expr) => {
        fn $do_name(&self, alias: &str) -> Result;
        fn $run_name(&self, arg_count: usize, args: Vec<&str>) -> Result {
            debug!("Command {} run.",stringify!($func_name));
            match arg_count {
                0 => self.$do_name(args[0]),
                //1 => {
                    //self.do_$do_name(args[0], args[1])
                //},
                _ => Err(CmdError::TooManyArgs(1, arg_count)),
            }
        }
    }
}

macro_rules! make_run {
    ($( $command_alias:pat => $run_func:ident ),*) => {
    fn run(&self, args: Vec<&str>) -> Result {
        if args.len() == 0 {
            Err(CmdError::NothingGiven())
        } else {
            let args_count = args.len() - 1;
            match args[0] {
                $(
                    $command_alias => { self.$run_func(args_count, args) },
                )*
                _ => { Err(CmdError::NotFound(args[0].to_string())) }
            }
        }
    }
    };
}

pub trait CommandCentre {
    
    create_run_function!(run_ok, do_ok, 0, 1);

    make_run!(
        "help" => run_help,
        "?" => run_help,
        "play" => run_play,
        "ok" => run_ok
    );
    //fn run(&self, args: Vec<&str>) -> Result {
    //    let len = args.len();
    //    if len == 0 {
    //        return Err(CmdError::NothingGiven())
    //    } else {
    //        match args[0] {
    //            "help" => { self.run_help(args.len() - 1, args) },
    //            "?" => { self.run_help(args.len() - 1, args) },
    //            "play" => { self.run_play(args.len() - 1, args) },
    //            "ok" => { self.run_ok(args.len() - 1, args) },
    //            "msg" => { self.run_msg(args.len() - 1, args) },
    //            _ => { Err(CmdError::NotFound(args[0].to_string())) }
    //        }
    //    }
    //}

    fn run_help(&self, arg_count: usize, args: Vec<&str>) -> Result {
        trace!("Command 'help' run.");
        match arg_count {
            0 => self.do_help(args[0]),
            1 => {
                match args[1].parse::<usize>() {
                    Ok(page) => self.do_help_page(args[0], page),
                    Err(_) => self.do_help_command(args[0], args[1]),
                }
            },
            _ => Err(CmdError::TooManyArgs(1, arg_count)),
        }

    }
    fn run_play(&self, arg_count: usize, args: Vec<&str>) -> Result {
        trace!("Command 'play' run.");
        Ok("play".to_string())
    }
    fn run_msg(&self, arg_count: usize, args: Vec<&str>) -> Result {
        trace!("Command 'msg' run.");
        match arg_count {
            0|1 => Err(CmdError::NotEnoughArgs{needs: 2, got: 1}),
            2 => {
                self.do_msg(args[0], args[1], args[2])
            },
            _ => Err(CmdError::TooManyArgs(1, arg_count)),
        }
    }    
    // Gets general help
    fn do_help(&self, alias: &str) -> Result;
    // Gets help for given command.
    fn do_help_command(&self, alias: &str, command: &str) -> Result;
    // Gets help at page #
    fn do_help_page(&self, alias: &str, page: usize) -> Result;

    // Gets general help
    fn do_msg(&self, alias: &str, to: &str, message: &str) -> Result;
}
