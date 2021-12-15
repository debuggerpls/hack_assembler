use std::fs;
use std::error::Error;
use std::collections::HashMap;
use std::ops::Add;

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
    let mut assembler = HackAssembler::new(&config);
    let mut parser = Parser::new(&config)?;
    let mut symbols = SymbolTable::new();

    // First pass
    loop {
        match parser.instruction_type() {
            Some(Instruction::L) => {
                // add to the symbol table
                symbols.add_entry(parser.symbol().unwrap(), parser.current_instruction as i32);
                // remove that line, so further symbols match the lines
                if parser.has_more_lines() {
                    // do not advance here!
                    parser.lines.remove(parser.current_instruction);
                    continue;
                } else {
                    parser.lines.remove(parser.current_instruction);
                    break;
                }

            }
            // Some(Instruction::A) => {
            //
            //     match parser.symbol().unwrap().parse::<i32>() {
            //         Ok(num) => {
            //             let binary = format!("{:016b}", num);
            //             // println!("{}", s);
            //             assembler.add_bytecode(&binary);
            //         },
            //         _ => println!("Unknown yet"),
            //     }
            // },
            // Some(Instruction::C) => {
            //     let mut binary = String::from("111");
            //     binary += &Code::comp(parser.comp());
            //     binary += &Code::dest(parser.dest());
            //     binary += &Code::jump(parser.jump());
            //     // println!("{}", binary);
            //     assembler.add_bytecode(&binary);
            // }
            _ => (),
        }

        if !parser.has_more_lines() {
            break;
        }

        parser.advance();
    }

    // reset parser
    parser.current_instruction = 0;

    // Second pass
    loop {
        match parser.instruction_type() {
            Some(Instruction::A) => {

                match parser.symbol().unwrap().parse::<i32>() {
                    Ok(num) => {
                        let binary = format!("{:016b}", num);
                        // println!("{}", s);
                        assembler.add_bytecode(&binary);
                    },
                    _ => println!("Unknown yet"),
                }
            },
            Some(Instruction::C) => {
                let mut binary = String::from("111");
                binary += &Code::comp(parser.comp());
                binary += &Code::dest(parser.dest());
                binary += &Code::jump(parser.jump());
                // println!("{}", binary);
                assembler.add_bytecode(&binary);
            }
            _ => (),
        }

        if !parser.has_more_lines() {
            break;
        }

        parser.advance();
    }

    assembler.write_to_file()?;

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
            Some(Instruction::A) => match &line[1..] {
                "R0" => Some("0".to_string()),
                "R1" => Some("1".to_string()),
                "R2" => Some("2".to_string()),
                "R3" => Some("3".to_string()),
                "R4" => Some("4".to_string()),
                "R5" => Some("5".to_string()),
                "R6" => Some("6".to_string()),
                "R7" => Some("7".to_string()),
                "R8" => Some("8".to_string()),
                "R9" => Some("9".to_string()),
                "R10" => Some("10".to_string()),
                "R11" => Some("11".to_string()),
                "R12" => Some("12".to_string()),
                "R13" => Some("13".to_string()),
                "R14" => Some("14".to_string()),
                "R15" => Some("15".to_string()),
                "SCREEN" => Some("16384".to_string()),
                "KBD" => Some("24576".to_string()),
                "SP" => Some("0".to_string()),
                "LCL" => Some("1".to_string()),
                "ARG" => Some("2".to_string()),
                "THIS" => Some("3".to_string()),
                "THAT" => Some("4".to_string()),
                _ => Some(line[1..].to_string()),
            },
            Some(Instruction::L) => {
                let matches: &[_] = &['(', ')'];
                Some(line.trim_matches(matches).to_string())
            }
            _ => None,
        }
    }

    fn dest(&self) -> Option<String> {
        let line = &self.lines[self.current_instruction];
        match self.instruction_type() {
            Some(Instruction::C) => {
                match line.find('=') {
                    Some(pos) => Some(line[..pos].to_string()),
                    None => None,
                }
            }
            _ => None,
        }
    }

    fn comp(&self) -> Option<String> {
        let line = &self.lines[self.current_instruction];
        match self.instruction_type() {
            Some(Instruction::C) => {
                let start = match line.find('=') {
                    Some(pos) => pos + 1,
                    None => 0,
                };
                let end = line.find(';');
                if end.is_some() {
                    let end = end.unwrap();
                    Some(line[start..end].to_string())
                } else {
                    Some(line[start..].to_string())
                }
            }
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
            }
            _ => None,
        }
    }
}

struct Code;

impl Code {
    fn dest(dest: Option<String>) -> String {
        match dest {
            None => String::from("000"),
            Some(d) => match &d[..] {
                "M" => String::from("001"),
                "D" => String::from("010"),
                "DM" => String::from("011"),
                "A" => String::from("100"),
                "AM" => String::from("101"),
                "AD" => String::from("110"),
                "ADM" => String::from("111"),
                _ => panic!("Invalid dest: {}", d),
            }
        }
    }

    fn jump(jump: Option<String>) -> String {
        match jump {
            None => String::from("000"),
            Some(cond) => match &cond[..] {
                "JGT" => String::from("001"),
                "JEQ" => String::from("010"),
                "JGE" => String::from("011"),
                "JLT" => String::from("100"),
                "JNE" => String::from("101"),
                "JLE" => String::from("110"),
                "JMP" => String::from("111"),
                _ => panic!("Invalid jump condition: {}", cond),
            }
        }
    }

    fn comp(comp: Option<String>) -> String {
        match comp {
            None => panic!("No comp provided!"),
            Some(comp) => match &comp[..] {
                "0" => "0101010".to_string(),
                "1" => "0111111".to_string(),
                "-1" => "0111010".to_string(),
                "D" => "0001100".to_string(),
                "A" => "0110000".to_string(),
                "!D" => "0001101".to_string(),
                "!A" => "0110011".to_string(),
                "-D" => "0001111".to_string(),
                "-A" => "0110011".to_string(),
                "D+1" => "0011111".to_string(),
                "A+1" => "0110111".to_string(),
                "D-1" => "0001110".to_string(),
                "A-1" => "0110010".to_string(),
                "D+A" => "0000010".to_string(),
                "D-A" => "0010011".to_string(),
                "A-D" => "0000111".to_string(),
                "D&A" => "0000000".to_string(),
                "D|A" => "0010101".to_string(),
                "M" => "1110000".to_string(),
                "!M" => "1110001".to_string(),
                "-M" => "1110011".to_string(),
                "M+1" => "1110111".to_string(),
                "M-1" => "1110010".to_string(),
                "D+M" => "1000010".to_string(),
                "D-M" => "1010011".to_string(),
                "M-D" => "1000111".to_string(),
                "D&M" => "1000000".to_string(),
                "D|M" => "1010101".to_string(),
                _ => panic!("Invalid comp: {}", comp),
            }
        }
    }
}

struct SymbolTable {
    symbols: HashMap<String, i32>,
}

impl SymbolTable {
    fn new() -> SymbolTable {
        SymbolTable {
            symbols: HashMap::new(),
        }
    }

    fn contains(&self, symbol: &str) -> bool {
        self.symbols.contains_key(symbol)
    }

    fn add_entry(&mut self, symbol: String, address: i32) {
        self.symbols.insert(symbol, address);
    }

    fn get_address(&self, symbol: &str) -> Option<&i32> {
        self.symbols.get(symbol)
    }
}

struct HackAssembler {
    output_file: String,
    bytecode: String,
}

impl HackAssembler {
    fn new(config: &Config) -> HackAssembler {
        HackAssembler {
            output_file: config.output_file.clone(),
            bytecode: String::new(),
        }
    }

    fn write_to_file(&self) -> Result<(), Box<dyn Error>> {
        fs::write(self.output_file.clone(), self.bytecode.clone())?;

        Ok(())
    }

    fn add_bytecode(&mut self, bytecode: &str) -> Result<(), String> {
        if bytecode.len() != 16 {
            return Err("Wrong size, should be 16 chars!".to_string());
        }

        self.bytecode += bytecode;
        self.bytecode += "\n";

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_create() {
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
    fn test_instruction_types() {
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
    fn test_instruction_symbols() {
        let contents = String::from("\
@2
@sum
D=0
@R2
@R15
@SCREEN
@KBD
@SP
@LCL
@ARG
@THIS
@THAT
(END)");

        let mut parser = Parser::create(contents);

        assert_eq!(parser.symbol(), Some("2".to_string()));
        parser.advance();
        assert_eq!(parser.symbol(), Some("sum".to_string()));
        parser.advance();
        assert_eq!(parser.symbol(), None);
        parser.advance();
        assert_eq!(parser.symbol(), Some("2".to_string()));
        parser.advance();
        assert_eq!(parser.symbol(), Some("15".to_string()));
        parser.advance();
        assert_eq!(parser.symbol(), Some("16384".to_string()));
        parser.advance();
        assert_eq!(parser.symbol(), Some("24576".to_string()));
        parser.advance();
        assert_eq!(parser.symbol(), Some("0".to_string()));
        parser.advance();
        assert_eq!(parser.symbol(), Some("1".to_string()));
        parser.advance();
        assert_eq!(parser.symbol(), Some("2".to_string()));
        parser.advance();
        assert_eq!(parser.symbol(), Some("3".to_string()));
        parser.advance();
        assert_eq!(parser.symbol(), Some("4".to_string()));
        parser.advance();
        assert_eq!(parser.symbol(), Some("END".to_string()));
    }

    #[test]
    fn test_dest_comp_jump() {
        let contents = String::from("\
@2
@sum
D=0
D=D+1;JLE
D;JGT
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
        assert_eq!(parser.dest(), None);
        assert_eq!(parser.comp(), Some("D".to_string()));
        assert_eq!(parser.jump(), Some("JGT".to_string()));
        parser.advance();
        assert!(parser.dest().is_none() && parser.comp().is_none() && parser.jump().is_none());
    }

    #[test]
    #[should_panic(expected = "Invalid dest")]
    fn test_code_dest() {
        assert_eq!(Code::dest(None), "000");
        assert_eq!(Code::dest(Some(String::from("M"))), "001");
        assert_eq!(Code::dest(Some(String::from("D"))), "010");
        assert_eq!(Code::dest(Some(String::from("DM"))), "011");
        assert_eq!(Code::dest(Some(String::from("A"))), "100");
        assert_eq!(Code::dest(Some(String::from("AM"))), "101");
        assert_eq!(Code::dest(Some(String::from("AD"))), "110");
        assert_eq!(Code::dest(Some(String::from("ADM"))), "111");

        // panic
        Code::dest(Some(String::from("ELMO")));
    }

    #[test]
    #[should_panic(expected = "Invalid jump condition")]
    fn test_code_jump() {
        assert_eq!(Code::jump(None), "000");
        assert_eq!(Code::jump(Some(String::from("JGT"))), "001");
        assert_eq!(Code::jump(Some(String::from("JEQ"))), "010");
        assert_eq!(Code::jump(Some(String::from("JGE"))), "011");
        assert_eq!(Code::jump(Some(String::from("JLT"))), "100");
        assert_eq!(Code::jump(Some(String::from("JNE"))), "101");
        assert_eq!(Code::jump(Some(String::from("JLE"))), "110");
        assert_eq!(Code::jump(Some(String::from("JMP"))), "111");

        // panic
        Code::jump(Some(String::from("ELMO")));
    }

    #[test]
    fn test_code_comp() {
        assert_eq!(Code::comp(Some(String::from("0"))), "0101010");
        assert_eq!(Code::comp(Some(String::from("1"))), "0111111");
        assert_eq!(Code::comp(Some(String::from("-1"))), "0111010");
        assert_eq!(Code::comp(Some(String::from("D"))), "0001100");
        assert_eq!(Code::comp(Some(String::from("A"))), "0110000");
        assert_eq!(Code::comp(Some(String::from("!D"))), "0001101");
        assert_eq!(Code::comp(Some(String::from("!A"))), "0110011");
        assert_eq!(Code::comp(Some(String::from("-D"))), "0001111");
        assert_eq!(Code::comp(Some(String::from("-A"))), "0110011");
        assert_eq!(Code::comp(Some(String::from("D+1"))), "0011111");
        assert_eq!(Code::comp(Some(String::from("A+1"))), "0110111");
        assert_eq!(Code::comp(Some(String::from("D-1"))), "0001110");
        assert_eq!(Code::comp(Some(String::from("A-1"))), "0110010");
        assert_eq!(Code::comp(Some(String::from("D+A"))), "0000010");
        assert_eq!(Code::comp(Some(String::from("D-A"))), "0010011");
        assert_eq!(Code::comp(Some(String::from("A-D"))), "0000111");
        assert_eq!(Code::comp(Some(String::from("D&A"))), "0000000");
        assert_eq!(Code::comp(Some(String::from("D|A"))), "0010101");
        assert_eq!(Code::comp(Some(String::from("M"))), "1110000");
        assert_eq!(Code::comp(Some(String::from("!M"))), "1110001");
        assert_eq!(Code::comp(Some(String::from("-M"))), "1110011");
        assert_eq!(Code::comp(Some(String::from("M+1"))), "1110111");
        assert_eq!(Code::comp(Some(String::from("M-1"))), "1110010");
        assert_eq!(Code::comp(Some(String::from("D+M"))), "1000010");
        assert_eq!(Code::comp(Some(String::from("D-M"))), "1010011");
        assert_eq!(Code::comp(Some(String::from("M-D"))), "1000111");
        assert_eq!(Code::comp(Some(String::from("D&M"))), "1000000");
        assert_eq!(Code::comp(Some(String::from("D|M"))), "1010101");
    }

    #[test]
    #[should_panic(expected = "No comp provided")]
    fn test_code_comp_panic1() {
        Code::comp(None);
    }

    #[test]
    #[should_panic(expected = "Invalid comp")]
    fn test_code_comp_panic2() {
        Code::comp(Some(String::from("ELMO")));
    }

    #[test]
    fn test_symboltable() {
        let mut symbols = SymbolTable::new();

        assert!(!symbols.contains("END"));
        symbols.add_entry("END".to_string(), 123);
        assert!(symbols.contains("END"));
        assert_eq!(symbols.get_address("END"), Some(&123));
        assert_eq!(symbols.get_address("START"), None);
    }
}