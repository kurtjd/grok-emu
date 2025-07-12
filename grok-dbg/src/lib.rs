use std::{collections::HashSet, io::Write};

fn addr_from_str(addr: &str) -> Result<usize, std::num::ParseIntError> {
    let addr = addr.trim_start_matches("0x");
    usize::from_str_radix(addr, 16)
}

pub trait DebugHandler {
    fn step(&mut self) -> usize;
    fn print_debug(&mut self);
    fn peek(&mut self, addr: usize) -> u8;
}

pub struct Debugger<T: DebugHandler> {
    target: T,
    brk: HashSet<usize>,
    exit: bool,
}

impl<T: DebugHandler> Debugger<T> {
    pub fn new(dbg: T) -> Self {
        Self {
            target: dbg,
            brk: HashSet::new(),
            exit: false,
        }
    }

    pub fn start(&mut self) {
        println!("grok-dbg");
        println!("© Grok the planet!\n");
        println!("Enter 'help' for a list of commands\n");

        while !self.exit {
            self.target.print_debug();
            let words = self.get_input();
            self.process_input(words);
            println!();
        }
    }

    fn set_bp(&mut self, addr: usize) {
        self.brk.insert(addr);
    }

    fn clr_bp(&mut self, addr: usize) {
        self.brk.remove(&addr);
    }

    fn continue_bp(&mut self) {
        loop {
            let addr = self.target.step();
            if self.brk.contains(&addr) {
                break;
            }
        }
    }

    fn get_input(&mut self) -> Vec<String> {
        print!("> ");
        std::io::stdout().flush().unwrap();

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        input
            .split_ascii_whitespace()
            .map(|s| s.to_string())
            .collect::<Vec<String>>()
    }

    fn process_input(&mut self, words: Vec<String>) {
        let words: Vec<&str> = words.iter().map(|s| s.as_str()).collect();
        if words.is_empty() {
            return;
        }

        match words.as_slice() {
            ["help"] => self.display_help(),
            ["setbp", addr] | ["bp", addr] | ["break", addr] => {
                if let Ok(addr) = addr_from_str(addr) {
                    self.set_bp(addr);
                }
            }
            ["clrbp", addr] => {
                if let Ok(addr) = addr_from_str(addr) {
                    self.clr_bp(addr);
                }
            }
            ["showbp"] => todo!(),
            ["peek", addr] => {
                if let Ok(addr) = addr_from_str(addr) {
                    println!("{:04X}={:02X}", addr, self.target.peek(addr));
                }
            }
            ["step"] | ["s"] | ["next"] | ["n"] => {
                self.target.step();
            }
            ["continue"] | ["c"] => {
                self.continue_bp();
            }
            ["exit"] => {
                self.exit = true;
            }
            _ => println!("Unrecognized command"),
        }
    }

    fn display_help(&self) {
        println!("help: Display help menu");
        println!("setbp <hex addr>: Set breakpoint at <addr>");
        println!("clrbp <hex addr>: Clear breakpoint at <addr>");
        println!("showbp: Display active breakpoints");
        println!("peek <hex addr>: Display byte at memory <addr>");
        println!("step: Step one instruction");
        println!("continue: Continue program execution until breakpoint");
        println!("exit: Exit debugger");
    }
}
