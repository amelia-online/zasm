use regex::Regex;
use std::io::{Read, Write};
use std::u16;
use std::collections::HashMap;
mod zasmenv;

#[derive(Debug)]
struct Instruction {
    opcode: u8,
    r1: u8,
    r2: u8,
    imm: u16,
}

#[derive(Debug, PartialEq, Clone)]
enum Param {
    Immediate(u16),
    Register(u8),
    Deref(Box<Param>),
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

    pub fn from_params(opcode: u8, p1: Param, p2: Param) -> Self {
	use Param::*;
	Self {
	    opcode: opcode,
	    r1: match p1 {
		Register(n) => n,
		_ => 0,
	    },
	    r2: match p2 {
		Register(n) => n,
		_ => 0,
	    },
	    imm: match p2 {
		Immediate(n) => n,
		_ => 0,
	    },
	    
	}
    }

    pub fn to_bytes(&self) -> [u8; 4] {
	let b1 = self.opcode;
	let b2 = (self.r1 << 4) | self.r2;
	let b3: u8 = ((self.imm >> 8) & 0xFF).try_into().unwrap();
	let b4 = (self.imm & 0xFF).try_into().unwrap();
	[b1, b2, b3, b4]
    }
}

fn parse_dbl(syntax: String, pat: &Regex) -> Option<Instruction> {
    let caps = pat.captures(syntax.as_str()).unwrap();
    let instr = &caps["i"];
    let str_p1 = &caps["p1"];
    let str_p2 = &caps["p2"];

    let opt_p1 = parse_param(str_p1.to_string());
    let opt_p2 = parse_param(str_p2.to_string());

    if opt_p1.is_none() || opt_p2.is_none() {
		return None;
    }

    let p1 = opt_p1.unwrap();
    let p2 = opt_p2.unwrap();

    if let Some(res_instr) = parse_instr(instr.to_string(), &p1, &p2) {
	Some(Instruction::from_params(res_instr, p1, p2))
    } else {
	None
    }
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

fn match_arithmetic(p1: &Param, p2: &Param, rr: u8, rd: u8, ri: u8) -> Option<u8> {
    use Param::*;
    match p1 {
	Register(_) => {
	    match p2 {
		Register(_) => Some(rr),
		Deref(_) => Some(rd),
		Immediate(_) => Some(ri),
	    }
	},	
	_ => None,
    }
}

fn match_bitwise(p1: &Param, p2: &Param, rr: u8, ri: u8) -> Option<u8> {
    use Param::*;
    match p1 {
	Register(_) => {
	    match p2 {
		Register(_) => Some(rr),
		Immediate(_) => Some(ri),
		_ => None,
	    }
	},
	_ => None,
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
	"add" => match_arithmetic(p1, p2, 0x27, 0x28, 0x29),
	"sub" => match_arithmetic(p1, p2, 0x2A, 0x2B, 0x2C),
	"mult" => match_arithmetic(p1, p2, 0x2D, 0x2E, 0x2F),
	"div" => match_arithmetic(p1, p2, 0x30, 0x31, 0x32),
	"mod" => match_arithmetic(p1, p2, 0x33, 0x34, 0x35),
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
	"and" => match_bitwise(p1, p2, 0x38, 0x39),
	"or" => match_bitwise(p1, p2, 0x3A, 0x3B),
	"xor" => match_bitwise(p1, p2, 0x3C, 0x3D),
	"not" => Some(0x3E),
	"read" => None, // TBD
	"writei" => Some(0x40),
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

fn parse_sngl(syntax: String, sap: &Regex, labels: &HashMap<String, u16>) -> Option<Instruction> {
    let caps = sap.captures(syntax.as_str()).unwrap();
    let instr = (&caps["i"]).to_string();
    let infoi = instr.clone();
    let mut str_p1 = (&caps["p1"]).to_string();
    if labels.contains_key(&str_p1) {
	str_p1 = String::from(format!("{}", labels.get(&str_p1).unwrap()));
    }
    if let Some(p1) = parse_param(str_p1) {
	if let Some(res_instr) = parse_instr(instr, &p1, &Param::Immediate(0)) {
	    let p2 = p1.clone(); // Temp solution
	    Some(Instruction::from_params(res_instr, p1, p2))
	} else {
	    println!("Error: could not parse instruction: '{}'", infoi);
	    None
	}
    } else {
	println!("Error: could not parse parameter.");
	None
    }
}

fn parse(line: String, dblpat: &Regex, snglpat: &Regex, labels: &HashMap<String, u16>) -> Option<Instruction> {
    if dblpat.is_match(line.as_str()) {
	return parse_dbl(line, dblpat);
    } else if snglpat.is_match(line.as_str()) {
	return parse_sngl(line, snglpat, labels);
    } else {
	return None;
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
    let single_arg_pattern = Regex::new(r"^(?<i>\w+) (?<p1>[ -~]+)$").unwrap();
    let field_pat = Regex::new(r"^(?<n>\w+): (?<item>)").unwrap();
    let args = std::env::args().collect::<Vec<String>>();
    
    if args.len() < 2 {
	println!("Please provide a file path.");
	std::process::exit(0);
    }

    let mut output_name = String::from("a.out.zm");
 
    if let Ok(text) = file_to_string(args[1].to_owned()) {
	let mut output = std::fs::File::create(format!("./{}", output_name)).unwrap();
	let lines: Vec<String> = text
	    .split("\n")
	    .map(|s| s.to_string())
	    .collect();

	let mut labels: HashMap<String, u16> = HashMap::new();
	let mut instr_counter = 0;
	for (i, line) in lines.iter().enumerate() {

	    if line.is_empty() || line.starts_with(";") { // Primitive checking for comment.
		continue;
	    }

	    if line.ends_with(":") { // Primitive checking for label.
		let mut label = line.trim().to_owned();
		label.pop();
		labels.insert(label, instr_counter);
		continue;
	    }
	    
	    let linei = line.clone();
	    if let Some(instr) = parse(line.to_owned(), &instr_pattern, &single_arg_pattern, &labels) {
		instr_counter += 1;
		let bytes = instr.to_bytes();
		for byte in bytes {
		    let _ = output.write(&[byte]);
		}
	    } else {
		println!("Error on line {}: unrecognized instruction encountered: '{}'", i+1, linei);
		std::process::exit(0);
	    }
	}
    } else {
	println!("Could not open file `{}`", &args[1]);
	std::process::exit(0);
    }
    
    
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

#[test]
fn test_instr_to_bytes() {
    let instr = Instruction::from(0x21, 0x7, 0x8, 0x0);
    let actual = instr.to_bytes();
    assert_eq!(actual, [0x21, 0x78, 0x0, 0x0]);
}

#[test]
fn test_instr_to_bytes_imm() {
    let instr = Instruction::from(0x21, 0x0, 0x0, 0x2040);
    let bytes = instr.to_bytes();
    assert_eq!(bytes, [0x21, 0x0, 0x20, 0x40]);
}

#[test]
fn instr_from_params() {
    let instr = Instruction::from_params(0x21, Param::Register(0x7), Param::Immediate(0x2));
    let actual = instr.to_bytes();
    assert_eq!(actual, [0x21, 0x70, 0x0, 0x2]);
}

#[test]
fn parse_instr_cmp() {
    let instr_pattern = Regex::new(r"^(?<i>\w+) (?<p1>[ -~]+), (?<p2>[ -~]+)$").unwrap();
    let sngl_pat = Regex::new(r"^(?<i>\w+) (?<p1>[ -~]+)$").unwrap();
    let instr = parse("cmp @rg0, 0x21".to_string(), &instr_pattern,
		      &sngl_pat);
    assert!(instr.is_some());
    let bytes = instr.unwrap().to_bytes();
    assert_eq!(bytes, [0x20, 0x70, 0x00, 0x21]);
}

#[test]
fn parse2_instr() {
    let single_arg_pattern = Regex::new(r"^(?<i>\w+) (?<p1>[ -~]+)$").unwrap();
    let res = parse_sngl("pop8 @rg0".to_string(), &single_arg_pattern);
    assert!(res.is_some());
    assert_eq!(res.unwrap().to_bytes(), [0x18, 0x70, 0x0, 0x00]);
}
