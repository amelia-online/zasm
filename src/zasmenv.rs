pub mod zasmenv {

    use regex::Regex;

    pub struct Binding {
	ident: String,
	address: u16,
	bytes: Vec<u8>,
    }

    impl Binding {
	pub fn new(ident: String, addr: u16, bytes: Vec<u8>) -> Self {
	    Self {
		ident: ident,
		address: addr,
		bytes: bytes
	    }
	}
    }

    pub fn parse_data(section: String) -> Vec<Binding> {
	let pat = Regex::new(r"^(?<n>\w+): (?<i>[ -~]+)$").unwrap();
	let mut res: Vec<Binding> = Vec::new();
	for field in section.split("\n") {
	    if let Ok(binding) = parse_field(field.to_string(), &pat) {
		res.push(binding);
	    }
	}
	res
    }
    
    pub fn parse_field(field: String, pat: &Regex) -> Result<Binding, ()> {
	if !pat.is_match(field.as_str()) {
	    Err(())
	} else {
	    let caps = pat.captures(field.as_str()).unwrap();
	    
	   todo!()
	}
    }

    pub fn parse_entry(s: String) -> Result<Vec<u8>, ()> {
	todo!()
    }

    pub fn is_string() -> bool {
	todo!()
    }

    pub fn parse_string(s: String) -> Result<String, ()> {
	todo!()
    }
}
