use regex::Regex;
use std::io::{Read, Write};
use std::u16;

#[derive(Debug)]
struct Instruction {
    opcode: u8,
    r1: u8,
    r2: u8,
    imm: u16,
}

#[derive(Debug, PartialEq)]
enum Param {
    Immediate(u16),
    Register(u8),
    Deref(Box<Param>)
}

impl Instruction {
    pub fn new() -> Self {
	Self {
	    opcode: 0,
	    r1: 0,
	    r2: 0,
	    imm: 0,
	}
    }

    pub fn from(opcode: u8, r1: u8, r2: u8, imm: u16) -> Self {
	Self {
	    opcode: opcode,
	    r1: r1,
	    r2: r2,
	    imm: imm,
	}
    }
}

fn parse(syntax: String, pat: &Regex) -> Option<Instruction> {
    if !pat.is_match(syntax.as_str()) {
	return None;
    }
    let caps = pat.captures(syntax.as_str()).unwrap();
    let instr = &caps["i"];
    let p1 = &caps["p1"];
    let p2 = &caps["p2"];

    let res_p1 = parse_param(p1.to_string());
    let res_p2 = parse_param(p2.to_string());

    if res_p1.is_none() || res_p2.is_none() {
	return None;
    }

    let res_instr = parse_instr(instr.to_string(), &res_p1.unwrap(), &res_p2.unwrap());

    println!("{} - {} - {}", instr, p1, p2);
    return Some(Instruction::new());
}

fn match_copy(p1: &Param, p2: &Param, v1: u8, v2: u8, v3: u8, v4: u8) -> Option<u8> {
    use Param::*;
    match p1 {
	Deref(_) => {
	    match p2 {
		Register(_) => Some(v1),
		_ => None,
	    }
	},
	Register(_) => {
	    match p2 {
		Deref(_) => Some(v2),
		Register(_) => Some(v3),
		Immediate(_) => Some(v4),
	    }
	},
	Immediate(_) => None,
    }
}

fn parse_instr(syntax: String, p1: &Param, p2: &Param) -> Option<u8> {
    use Param::*;
    // I'm so sorry.
    match syntax.trim() {
	"writes" => {
	    match p1 {
		Deref(_)     => {
		    match p2 {
			Immediate(_) => Some(0),
			_ => None,
		    }
		},
		Register(_)  => {
		    match p2 {
			Immediate(_) => Some(1),
			_ => None,
		    }
		},
		_ => None,
	    }
	},
	"write" => Some(0x2),
	"copy8" => match_copy(p1, p2, 0x10, 0xC, 0x4, 0x8),
	"copy16" => match_copy(p1, p2, 0x11, 0xD, 0x5, 0x9),
	"copy32" => match_copy(p1, p2, 0x12, 0xE, 0x6, 0xA),
	"copy64" => match_copy(p1, p2, 0x13, 0xF, 0x7, 0xB),
	"push8" => Some(0x14),
	"push16" => Some(0x15),
	"push32" => Some(0x16),
	"push64" => Some(0x17),
	"pop8" => Some(0x18),
	"pop16" => Some(0x19),
	"pop32" => Some(0x1A),
	"pop64" => Some(0x1B),
	"goto" => Some(0x1C),
	"call" => Some(0x1D),
	"sys" => Some(0x1E),
	"cmp" => {
	    match p1 {
		Register(_) => {
		    match p2 {
			Register(_) => Some(0x1F),
			Immediate(_) => Some(0x20),
			_ => None,
		    }
		},
		_ => None,
	    }
	},
	"jge" => Some(0x21),
	"jg" => Some(0x22),
	"jle" => Some(0x23),
	"jl" => Some(0x24),
	"je" => Some(0x25),
	"jne" => Some(0x26),
	"add" => {
	    match p1 {
		Register(_) => {
		    match p2 {
			Register(_) => Some(0x27),
			Deref(_) => Some(0x28),
			Immediate(_) => Some(0x29),
		    }
		},

		_ => None,
	    }
	},
	
	"sub" => {
	    match p1 {
		Register(_) => {
		    match p2 {
			Register(_) => Some(0x2A),
			Deref(_) => Some(0x2B),
			Immediate(_) => Some(0x2C),
		    }
		},

		_ => None,
	    }
	},
	
	"mult" => {
	    match p1 {
		Register(_) => {
		    match p2 {
			Register(_) => Some(0x2D),
			Deref(_) => Some(0x2E),
			Immediate(_) => Some(0x2F),
		    }
		},

		_ => None,
	    }
	},
	
	"div" => {
	    match p1 {
		Register(_) => {
		    match p2 {
			Register(_) => Some(0x30),
			Deref(_) => Some(0x31),
			Immediate(_) => Some(0x32),
		    }
		},

		_ => None,
	    }
	},
	
	"mod" => {
	    match p1 {
		Register(_) => {
		    match p2 {
			Register(_) => Some(0x33),
			Deref(_) => Some(0x34),
			Immediate(_) => Some(0x35),
		    }
		},

		_ => None,
	    }
	},
	"shr" => {
	    match p1 {
		Register(_) => {
		    match p2 {
			Immediate(_) => Some(0x36),
			Register(_) => Some(0x42),
			_ => None,
		    }
		},
		_ => None,
	    }
	},
	"shl" => {
	    match p1 {
		Register(_) => {
		    match p2 {
			Immediate(_) => Some(0x37),
			Register(_) => Some(0x43),
			_ => None,
		    }
		},
		_ => None,
	    }
	},
	"and" => {
	    match p1 {
		Register(_) => {
		    match p2 {
			Register(_) => Some(0x38),
			Immediate(_) => Some(0x39),
			_ => None,
		    }
		},
		_ => None,
	    }
	},
	"or" => {
	    match p1 {
		Register(_) => {
		    match p2 {
			Register(_) => Some(0x3A),
			Immediate(_) => Some(0x3B),
			_ => None,
		    }
		},
		_ => None,
	    }
	},
	"xor" => {
	    match p1 {
		Register(_) => {
		    match p2 {
			Register(_) => Some(0x3C),
			Immediate(_) => Some(0x3D),
			_ => None,
		    }
		},
		_ => None,
	    }
	},
	"not" => Some(0x3E),
	"read" => None, // TBD
 	_ => None,
    }
}

fn parse_hex(s: String) -> Option<u16> {
    if let Ok(res) = u16::from_str_radix(s.as_str(), 16) {
	Some(res)
    } else {
	None
    }
}

fn parse_param(syntax: String) -> Option<Param> {
    use Param::*;

    if is_deref(&syntax) {
	if let Some(inner) = parse_param(strip_deref(&syntax)) {
	    return Some(Deref(Box::new(inner)));
	} else {
	    return None;
	}
    }

    if syntax.starts_with("@") {
	if let Some(reg) = parse_reg(&syntax) {
	    return Some(Register(reg));
	} else {
	    return None;
	}
    }

    if syntax.starts_with("0x") {
	if let Some(imm) = parse_hex(syntax.strip_prefix("0x").unwrap().to_string()) {
	    return Some(Immediate(imm));
	} else {
	    return None;
	}
    }

    if let Ok(num) = syntax.parse::<u16>() {
	return Some(Immediate(num))
    }
    
    return None;
}

fn strip_deref(s: &String) -> String {
    String::from(&s[1..s.len()-1])
}

fn is_deref(s: &String) -> bool {
    if s.starts_with("[") && s.ends_with("]") {
	true
    } else {
	false
    }
}

fn parse_reg(syntax: &String) -> Option<u8>{
    match syntax.trim() {
	"@ret" => Some(0x1),
	"@spr" => Some(0x2),
	"@glb" => Some(0x3),
	"@rta" => Some(0x4),
	"@fla" => Some(0x5),
	"@fpr" => Some(0x6),
	"@rg0" => Some(0x7),
	"@rg1" => Some(0x8),
	"@rg2" => Some(0x9),
	"@rg3" => Some(0xA),
	"@rg4" => Some(0xB),
	"@pr0" => Some(0xC),
	"@pr1" => Some(0xD),
	"@pr2" => Some(0xE),
	"@rg5" => Some(0xF),
	_ => None,
    }
}

fn file_to_string(path: String) -> Result<String, ()> {
    if let Ok(mut file) = std::fs::File::open(path) {
	let mut buf = String::new();
	file.read_to_string(&mut buf).unwrap();
	Ok(buf)
    } else {
	Err(())
    }
}

fn main() {
    let instr_pattern = Regex::new(r"^(?<i>\w+) (?<p1>[ -~]+), (?<p2>[ -~]+)$").unwrap();
    let args = std::env::args().collect::<Vec<String>>();
    parse("copy8 [@rg0], 1".to_string(), &instr_pattern);
}

// Tests start here.

#[test]
fn test_parse_reg() {
    let byte = parse_reg(&"@rg1".to_string());
    assert_eq!(byte, Some(0x8));
}

#[test]
fn test_strip_deref() {
    let s = String::from("[@rg1]");
    let actual = strip_deref(&s);
    assert_eq!(actual, "@rg1".to_string());
}

#[test]
fn test_is_deref() {
    assert!(is_deref(&"[@rg0]".to_string()));
}

#[test]
fn test_parse_param1() {
    use Param::*;
    let s = String::from("[@rg0]");
    let actual = parse_param(s);
    let expected = Some(Deref(Box::new(Register(0x7))));
    assert_eq!(actual, expected);
}

#[test]
fn test_parse_param2() {
    use Param::*;
    let s = String::from("@ret");
    let actual = parse_param(s);
    let expected = Some(Register(1));
    assert_eq!(actual, expected);
}

#[test]
fn test_parse_param3() {
    use Param::*;
    let s = String::from("[[12]]");
    let actual = parse_param(s);
    let expected = Some(Deref(Box::new(Deref(Box::new(Immediate(12))))));
    assert_eq!(actual, expected);
}

#[test]
fn test_parse_with_hex() {
    let actual = parse_param("0xAC".to_string());
    let expected = Some(Param::Immediate(0xAC));
    assert_eq!(actual, expected);
}
