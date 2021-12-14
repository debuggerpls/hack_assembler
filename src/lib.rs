use std::fs;
use std::error::Error;

pub struct Config {
    input_file: String,
    output_file: String,
}

impl Config {
    pub fn new(mut args: std::env::Args) -> Result<Config, &'static str> {
        args.next();

        let input_file = match args.next() {
            Some(arg) => arg,
            None => return Err("Didn't provide input file"),
        };

        let output_file = match args.next() {
            Some(arg) => arg,
            None => return Err("Didn't provide output file"),
        };

        Ok(Config { input_file, output_file })
    }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let parser = Parser::new(&config)?;

    for line in &parser.lines {
        println!("{}", line);
    }

    println!();

    Ok(())
}

struct Parser {
    lines: Vec<String>,
    current_instruction: usize,
}

#[derive(Debug)]
enum Instruction {
    A,
    C,
    L,
}

impl Parser {
    fn new(config: &Config) -> Result<Parser, Box<dyn Error>> {
        let source = fs::read_to_string(&config.input_file)?;

        Ok(Parser::create(source))
    }

    fn create(contents: String) -> Parser {
        let mut parser = Parser {
            lines: Vec::new(),
            current_instruction: 0,
        };

        parser.lines = contents
            .lines()
            .map(|line| {
                match line.find("//") {
                    Some(index) => &line[..index],
                    None => line
                }
            })
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .map(|line| line.to_string())
            .collect();

        parser
    }

    fn has_more_lines(&self) -> bool {
        self.lines.len() > self.current_instruction + 1
    }

    fn advance(&mut self) {
        self.current_instruction += 1;
    }

    // TODO: what if wrong line? should break, not just say its C_INSTRUCTION
    fn instruction_type(&self) -> Option<Instruction> {
        if self.current_instruction < self.lines.len() {
            let line = &self.lines[self.current_instruction];
            if line.starts_with('@') {
                Some(Instruction::A)
            } else if line.starts_with('(') && line.ends_with(')') {
                Some(Instruction::L)
            } else {
                Some(Instruction::C)
            }
        } else {
            None
        }
    }

    fn symbol(&self) -> Option<String> {
        let line = &self.lines[self.current_instruction];
        match self.instruction_type() {
            Some(Instruction::A) => Some(line[1..].to_string()),
            Some(Instruction::L) => {
                let matches: &[_] = &['(', ')'];
                Some(line.trim_matches(matches).to_string())
            },
            _ => None,
        }
    }

    fn dest(&self) -> Option<String> {
        let line = &self.lines[self.current_instruction];
        match self.instruction_type() {
            Some(Instruction::C) => {
                let end = line.find('=').unwrap();
                Some(line[..end].to_string())
            },
            _ => None,
        }
    }

    fn comp(&self) -> Option<String> {
        let line = &self.lines[self.current_instruction];
        match self.instruction_type() {
            Some(Instruction::C) => {
                let start = line.find('=').unwrap() + 1;
                let end = line.find(';');
                if end.is_some() {
                    let end = end.unwrap();
                    Some(line[start..end].to_string())
                } else {
                    Some(line[start..].to_string())
                }
            },
            _ => None,
        }
    }

    fn jump(&self) -> Option<String> {
        let line = &self.lines[self.current_instruction];
        match self.instruction_type() {
            Some(Instruction::C) => {
                let start = line.find(';');
                if start.is_some() {
                    let start = start.unwrap() + 1;
                    Some(line[start..].to_string())
                } else {
                    None
                }
            },
            _ => None,
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser_create() {
        let contents = String::from("\
// comment

@2
@3  // in-line comment");

        let mut parser = Parser::create(contents);

        assert_eq!(parser.lines, vec!["@2", "@3"]);
        assert!(parser.has_more_lines());
        parser.advance();
        assert!(!parser.has_more_lines());
    }

    #[test]
    fn instruction_types() {
        let contents = String::from("\
@2
@sum
D=0
(END)");

        let mut parser = Parser::create(contents);

        match parser.instruction_type() {
            Some(Instruction::A) => (),
            _ => panic!("Expected Instruction::A"),
        }
        parser.advance();
        match parser.instruction_type() {
            Some(Instruction::A) => (),
            _ => panic!("Expected Instruction::A"),
        }
        parser.advance();
        match parser.instruction_type() {
            Some(Instruction::C) => (),
            _ => panic!("Expected Instruction::C"),
        }
        parser.advance();
        match parser.instruction_type() {
            Some(Instruction::L) => (),
            _ => panic!("Expected Instruction::L"),
        }
    }

    #[test]
    fn instruction_symbols() {
        let contents = String::from("\
@2
@sum
D=0
(END)");

        let mut parser = Parser::create(contents);

        assert_eq!(parser.symbol(), Some("2".to_string()));
        parser.advance();
        assert_eq!(parser.symbol(), Some("sum".to_string()));
        parser.advance();
        assert_eq!(parser.symbol(), None);
        parser.advance();
        assert_eq!(parser.symbol(), Some("END".to_string()));
    }

    #[test]
    fn dest_comp_jump() {
        let contents = String::from("\
@2
@sum
D=0
D=D+1;JLE
(END)");

        let mut parser = Parser::create(contents);

        assert!(parser.dest().is_none() && parser.comp().is_none() && parser.jump().is_none());
        parser.advance();
        assert!(parser.dest().is_none() && parser.comp().is_none() && parser.jump().is_none());
        parser.advance();
        assert_eq!(parser.dest(), Some("D".to_string()));
        assert_eq!(parser.comp(), Some("0".to_string()));
        assert_eq!(parser.jump(), None);
        parser.advance();
        assert_eq!(parser.dest(), Some("D".to_string()));
        assert_eq!(parser.comp(), Some("D+1".to_string()));
        assert_eq!(parser.jump(), Some("JLE".to_string()));
        parser.advance();
        assert!(parser.dest().is_none() && parser.comp().is_none() && parser.jump().is_none());
    }
}