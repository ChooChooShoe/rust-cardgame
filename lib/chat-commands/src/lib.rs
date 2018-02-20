#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_macro() {
        let ex = ex {};
        assert_eq!(ex.run(vec!("ok")), Err(CmdError::NoPermission()));
    }

}

#[macro_use]
extern crate log;

use std::error;
use std::fmt;

pub type Result = std::result::Result<String, CmdError>;

#[derive(Debug, Clone,Eq,PartialEq)]
pub enum CmdError
{
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