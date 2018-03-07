//extern crate clap;
#[macro_use]
extern crate log;

mod parser;
mod permission;

//use clap::{Arg, App, SubCommand,AppSettings,ArgSettings};
use std::time::{Duration,Instant};
use parser::*;
use std::io::{self, Read,Write};

fn callback(cmd: &AppCommand, args: MatchedArgs) -> CmdResult {
    println!("cmd run");
    Err(CmdError::Generic("not implmented".to_string()))
}
fn main() {
    let t = Instant::now();

    //let external_sub_command = clap_app!( @subcommand message =>
    //    (author: "this")
    //    (@arg bar: -b "Bar")
    //);
    let mut centre = CommandCentre::new();

    centre.add(
        AppCommand::new("position")
            .alias("pos")
            .execute(callback)
    );
    centre.add(
        AppCommand::new("draw")
            .execute(callback)
            .arg(Arg::new("num", 0).typecheck(Type::PositveInt))
            
    );
    centre.add_help();
    loop {
        let mut input = String::new();
        io::stdout().write(b"> ").unwrap();
        io::stdout().flush().unwrap();
        match io::stdin().read_line(&mut input) {
            Ok(_n) => {
                //println!("read {} bytes /{}", n,input);
                match centre.run_from_input(&input) {
                    Ok(_) => println!("Complete"),
                    Err(e) => println!("Error: {}", e),
                }
            }
            Err(error) => println!("error: {}", error),
        }
    }
}
                //.help("The repo to clone")
                //.required(true)))
        //.subcommand(SubCommand::with_name("push")
        //    .about("pushes things")
        //    .setting(AppSettings::SubcommandRequiredElseHelp)
        //    .subcommand(SubCommand::with_name("remote")  // Subcommands can have thier own subcommands,
        //                                                 // which in turn have their own subcommands
        //        .about("pushes remote things")
        //        .arg(Arg::with_name("repo")
        //            .required(true)
        //            .help("The remote repo to push things to")))
        //    .subcommand(SubCommand::with_name("local")
        //        .about("pushes local things")))
        //.subcommand(SubCommand::with_name("add")
        //    .about("adds things")
        //    .author("Someone Else")                     // Subcommands can list different authors
        //    .version("v2.0 (I'm versioned differently") // or different version from their parents
        //    .setting(AppSettings::ArgRequiredElseHelp)  // They can even have different settings
        //    .arg(Arg::with_name("stuff")
        //        .long("stuff")
        //        .help("Stuff to add")
        //        .takes_value(true)
        //        .multiple(true)));
    //let matches = centre.app.get_matches();
//
    //// Gets a value for config if supplied by user, or defaults to "default.conf"
    //let config = matches.value_of("config").unwrap_or("default.conf");
    //println!("Value for config: {}", config);
//
    //// Calling .unwrap() is safe here because "INPUT" is required (if "INPUT" wasn't
    //// required we could have used an 'if let' to conditionally get the value)
    //println!("Using input file: {}", matches.value_of("INPUT").unwrap());
//
    //// Vary the output based on how many times the user used the "verbose" flag
    //// (i.e. 'myprog -v -v -v' or 'myprog -vvv' vs 'myprog -v'
    //match matches.occurrences_of("v") {
    //    0 => println!("No verbose info"),
    //    1 => println!("Some verbose info"),
    //    2 => println!("Tons of verbose info"),
    //    3 | _ => println!("Don't be crazy"),
    //}
//
    //// You can handle information about subcommands by requesting their matches by name
    //// (as below), requesting just the name used, or both at the same time
    //if let Some(matches) = matches.subcommand_matches("test") {
    //    if matches.is_present("debug") {
    //        println!("Printing debug info...");
    //    } else {
    //        println!("Printing normally...");
    //    }
    //}
//
    //let dur = t.elapsed();
    //println!("dur = {}",dur.as_secs() as f64 + dur.subsec_nanos() as f64 * 1e-9);
    //// more program logic goes here...

use std::collections::HashMap;
use std::rc::Rc;

fn help_callback(cmd: &AppCommand, args: MatchedArgs) -> CmdResult {
    println!("--- Help ---");
    Ok(())
}

pub struct CommandCentre{
    pub cmds: HashMap<String,Rc<AppCommand>>,
    pub last_cmds: Vec<String>,
}

impl CommandCentre {

    pub fn new() -> CommandCentre {
        CommandCentre {
            cmds: HashMap::new(),
            last_cmds: Vec::new(),
        }
    }
    pub fn add(&mut self, cmd: AppCommand) {
        let rc = Rc::new(cmd);
        for alias in rc.get_aliases() {
            self.cmds.insert(alias.to_string(), rc.clone());
        }
    }
    pub fn add_help(&mut self) {
        let help = AppCommand::new("help")
                    .alias("?")
                    .about("Get help for avalable commands")
                    .arg(Arg::new("page", 0).typecheck(Type::PositveInt))
                    .arg(Arg::new("command", 0).required(true))
                    .execute(help_callback);
        self.add(help);
    }
    
    pub fn run_from_input(&self, line: &str) -> CmdResult {
        let cmdline = CommandLine::from_line(line);
        if cmdline.is_invalid() {
            Err(CmdError::InvalidLine())
        } else if cmdline.args.len() == 0 {
            Err(CmdError::NothingGiven())
        } else {
            let args_count = cmdline.args.len() - 1;
            let args = cmdline.args();
            match self.cmds.get(args[0]) {
                Some(cmd) => {
                    (&cmd.callback)(&cmd, MatchedArgs {
                        alias: args[0],
                        args_count,
                        args: &args[1..]
                    })
                }
                None => Err(CmdError::NotFound(args[0].to_string())),
            }
        }
    }
}
