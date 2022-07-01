static HELP_STRNO: usize = 1;
static INSTRS_STRNO: usize = 2;

fn get_byte() -> u8 {
    use std::io::Read;

    std::io::stdin()
        .bytes() 
        .next()
        .and_then(|result| result.ok())
        .map(|x| x) // to convert to number do x-48 
        .unwrap()
}

#[derive(Debug, Eq, PartialEq)]
enum TokenType {
    Plus, Minus,
    Left, Right,
    Dot, Comma,
    Lpar, Rpar,
    Invalid
}

#[derive(Debug)]
struct Token {
    typ: TokenType,
    pos: usize
}

impl Token {
    pub fn new(typ: TokenType, pos: usize) -> Self {
        Self { typ, pos }
    }
}

fn lex(program: &String) -> Vec<Token> {
    let mut tokens = vec![];
    let mut id = 0;

    for ch in program.chars() {
        let token :Token;
        let typ = match ch {
            '+' => TokenType::Plus,
            '-' => TokenType::Minus,
            '<' => TokenType::Left,
            '>' => TokenType::Right,
            '.' => TokenType::Dot,
            ',' => TokenType::Comma,
            '[' => TokenType::Lpar,
            ']' => TokenType::Rpar,
            _ => TokenType::Invalid
        };
        token = Token::new(typ, id);

        if token.typ == TokenType::Invalid {
           panic!("Invalid token {:?} at {:?}", ch, id);
        } else {
            tokens.push(token);
        }
        id += 1;
    }

    tokens
}

fn seek_closing(start_id: usize, program: &Vec<Token>) -> Result<usize, String> {
    for t in program {
        if t.pos <= start_id { continue; }
        if t.typ == TokenType::Lpar {
            return Err(format!("Nested loops are not allowed. Position: {}", t.pos+1));
        }

        if t.typ == TokenType::Rpar {
            return Ok(t.pos);
        }
    }

    Ok(0) // no way a closing ] will be at the start of the program
}

fn parse(program: &Vec<Token>, capacity: usize, debug: bool, tape_debug: (bool, bool)) -> Result<(), String> {
    let mut tape = vec![0 as i32; capacity];
    let mut dp = 0; // data pointer
    let mut ip = 0; // index of the token within the program, instruction pointer
    let mut last_opening_pos = 0;

    loop {
        let token = &program[ip];

        match token.typ {
            TokenType::Left  => dp -= 1,
            TokenType::Right => dp += 1,
            TokenType::Plus  => tape[dp] += 1,
            TokenType::Minus => tape[dp] -= 1,
            TokenType::Comma => tape[dp] = get_byte() as i32,
            TokenType::Dot   => print!("{}", char::from_u32(tape[dp] as u32).unwrap() ),
            TokenType::Lpar  => {
                last_opening_pos = ip;
                if tape[dp] == 0 {
                    let next = seek_closing(ip, program)?;

                    if next == 0 {
                        return Err(format!("Closing ] not found for [ at {}", ip));
                    } else {
                        ip = next + 1;
                        continue;
                    }
                }
            },
            TokenType::Rpar  => {
                if tape[dp] != 0 {
                    ip = last_opening_pos + 1;
                    continue;
                }
            },
            _ => {}
        }

        ip += 1;

        if ip >= program.len() {
            break;
        }
    }

    if debug {
        let activate_tape = tape_debug.0;
        println!("
[INFO]
Debug information after execution:
    Data lane capacity: {}
    {}
        ", capacity, if activate_tape { "Tape cells:" } else { "" }
    );

        if activate_tape {
            let mut st = String::from("\t");
            for cell in tape {
                if !(tape_debug.1) && cell == 0 {
                    continue;
                } else {
                    st.push(cell.to_string().chars().next().unwrap());
                    st.push('|');
                }
            }
            println!("{}", st);
        }
    }
    
    Ok(())
}

fn puts(strno: usize) {
    let mut txt = "";

    txt = match strno {
        1 => { "
            Usage: rustfuck -f [FILENAME] {{OPTIONS}}

            Available options:
                -h --help:              Print this message
                -i -- instructions:     Print all available Brainfuck instructions
                -c --cap --capacity:    Change the capacity of the data storage. Default is 100.
                -f | --file [FILEPATH]: Provide a filename for the interpreter to use.
                -t | --tape {a | all}:  Print the tape. Will exclude all 0-valued cells by default. Provide value 'a' or 'all' to print all cells.        
            \n"
        },    
        2 => { "
            Available instructions:

            > 	Increment the data pointer (to point to the next cell to the right).
            < 	Decrement the data pointer (to point to the next cell to the left).
            + 	Increment (increase by one) the byte at the data pointer.
            - 	Decrement (decrease by one) the byte at the data pointer.
            . 	Output the byte at the data pointer.
            , 	Accept one byte of input, storing its value in the byte at the data pointer.
            [ 	If the byte at the data pointer is zero, then instead of moving the instruction pointer forward to the next command, jump it forward to the command after the matching ] command.
            ] 	If the byte at the data pointer is nonzero, then instead of moving the instruction pointer forward to the next command, jump it back to the command after the matching [ command.
            \n"
        },
        _ => {""}
    };

    print!("{}", txt);
}

fn exec(filename: &str, capacity: usize, debug: bool, tape_debug: (bool, bool)) -> Result<(), String> {
    let mut content: String = std::fs::read_to_string(filename).unwrap().parse().unwrap();

    content.retain(|x| !x.is_whitespace());

    parse(&lex(&content), capacity, debug, tape_debug)
}

fn main() -> Result<(), String> {

    let args = std::env::args().collect::<Vec<String>>();

    if args.len() == 1 {
        return Err("No arguments provided. See -h or --help for more info.".to_string());
    }

    let mut file = String::new();
    let mut cap: usize = 100;
    let mut i = 1;
    let mut debug = false;
    let mut tape_debug = (false, false); // (print tape, print all cells)

    loop {
        if args[i].starts_with('-') {
            match args[i].as_str() {
                "-h" | "--help" => {
                    puts(HELP_STRNO);
                    return Ok(());
                },
                "-i" | "--instructions" => {
                    puts(INSTRS_STRNO);
                    return Ok(());                
                },
                "-c" | "--cap" | "--capacity" => {
                    if let Ok(c) = args[i + 1].parse::<usize>() {
                        cap = c;
                    } else {
                        println!("[WARN] Please provide a valid number to {}. Running with default capacity.", args[i]);
                    }
                    i+=1;
                    continue;
                }
                "-f" | "--file" => {
                    if std::path::Path::exists(std::path::Path::new(args[i+1].as_str())) {
                        file = String::from(args[i+1].as_str());
                        i+=1;
                        continue;
                    } else {
                        return Err(format!("No file with filename \"{}\" was found", args[i+1]));  
                    }
                },
                "-d" | "--debug" => {
                    debug = true;
                },
                "-t" | "--tape" => {
                    tape_debug.0 = true;

                    if i < args.len() - 1 {
                        if args[i+1] == "a" || args[i + 1] == "all" {
                            tape_debug.1 = true;
                        }
                    }
                }
                _ => {
                    println!("Invalid argument \"{}\"", args[i]);
                }
            }
        }
        i+=1;

        if i == args.len() { break; }
    }
    
    println!("TAPE DBUG: {:?}", tape_debug);

    if file.is_empty() {
        return Err("No filename provided.".to_string())
    }

    exec(&file, cap, debug, tape_debug)
}
