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
}

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
