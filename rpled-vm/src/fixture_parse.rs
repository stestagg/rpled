use crate::sync::Sync;
use crate::vm::{NoVmDebug, VM};
use regex::{Regex, RegexSet};
use std::vec::Vec;

const OUTPUT_SEPARATOR: &str = "=== OUTPUT ===";

pub struct ParsedFixture {
    pub program: Vec<u8>,
    pub expected_output: String,
}

pub fn parse_fixture_with_output(data: &str) -> ParsedFixture {
    let (program_section, output_section) = data
        .rsplit_once(OUTPUT_SEPARATOR)
        .expect("Fixture must contain '=== OUTPUT ===' separator");

    ParsedFixture {
        program: decode_fixture(program_section),
        expected_output: output_section
            .trim()
            .lines()
            .collect::<Vec<&str>>()
            .join("\n"),
    }
}

pub fn decode_fixture(data: &str) -> Vec<u8> {
    // Each line is either:
    // - A blank line
    // - A double quote followed by characters (utf-8), ending with a double quote
    // - A series of numeric values (hex or decimal) separated by spaces
    //   - Hex values start with '0x' or '0X'
    //   - Hex values default to u8, or i16 if exactly 4 hex digits
    //   - Decimal values default to u8
    //   - Each value can have 'u8', 'u16', or 'i16' suffixes to indicate size and signedness
    // - OP:<OPNAME> [comma-separated arguments] - an opcode by name with optional arguments
    //   - Arguments can be hex (0xNN) or decimal numbers
    // - HEADER(XX) where XX is the heap size - expands to a valid header section
    // All values are parsed, and then concatenated into a single Vec<u8>.
    // Any line may have comments starting with '#', which should be ignored.
    // '#' within the double quotes of a quote line are treated as normal characters.

    let mut result: Vec<u8> = Vec::new();

    let quote_line_re = r#"^\s*"(?<quote>.*)"\s*(#.*)?$"#;
    let num_line_re = r"^(?<num>((0x|0X)?-?[0-9a-fA-F]+(u8|u16|i16)?\s*)+)(#.*)?$";
    let header_line_re = r"^\s*HEADER\((?<heap>\d+)\)\s*(#.*)?$";
    let op_line_re = r"^\s*OP:(?<opname>[A-Z0-9]+)\s*(?<args>[^#]*)(#.*)?$";
    let blank_line_re = r"^\s*(#.*)?$";

    let patterns = [
        quote_line_re,
        num_line_re,
        header_line_re,
        op_line_re,
        blank_line_re,
    ];

    let res = patterns
        .iter()
        .map(|p| Regex::new(p).unwrap())
        .collect::<Vec<Regex>>();

    let line_set = RegexSet::new(patterns).unwrap();

    for line in data.lines() {
        let matches = line_set.matches(line);
        let match_idx = match matches.iter().next() {
            Some(idx) => idx,
            None => {
                panic!("Line did not match any known pattern: {}", line);
            }
        };
        let capture = res[match_idx].captures(line).unwrap();
        if let Some(quote) = capture.name("quote") {
            let s = quote.as_str();
            result.extend_from_slice(s.as_bytes());
        }
        if let Some(num) = capture.name("num") {
            let s = num.as_str();
            let mut num_bytes = num_line_to_vec(s);
            result.append(&mut num_bytes);
        }
        if let Some(heap) = capture.name("heap") {
            let heap_size: u16 = heap.as_str().parse().expect("Failed to parse heap size");
            let mut header_bytes = generate_header(heap_size);
            result.append(&mut header_bytes);
        }
        if let Some(opname) = capture.name("opname") {
            let op_str = opname.as_str();
            let opcode = opcode_by_name::<crate::sync::TokioSync>(op_str)
                .unwrap_or_else(|| panic!("Unknown opcode: {}", op_str));
            result.push(opcode);

            if let Some(args) = capture.name("args") {
                let args_str = args.as_str().trim();
                if !args_str.is_empty() {
                    let mut arg_bytes = parse_op_args(args_str);
                    result.append(&mut arg_bytes);
                }
            }
        }
    }
    result
}

fn generate_header(heap_size: u16) -> Vec<u8> {
    let mut result = Vec::new();

    // Magic bytes: "PXS"
    result.extend_from_slice(b"PXS");

    // Version: 0
    result.push(0);

    // Heap size (u16, little-endian)
    result.extend_from_slice(&heap_size.to_le_bytes());

    // Remaining header length: 1 (num_modules) + 1 (module id) + 2 ("T1")
    result.push(4);

    // Number of modules: 1
    result.push(1);

    // Module: Test (60) with name "T1"
    result.push(60);
    result.extend_from_slice(b"T1");

    result
}

fn parse_number(token: &str) -> Vec<u8> {
    // Extract suffix if present
    let (num_str, suffix) = if token.ends_with("u8") {
        (&token[..token.len() - 2], Some("u8"))
    } else if token.ends_with("u16") {
        (&token[..token.len() - 3], Some("u16"))
    } else if token.ends_with("i16") {
        (&token[..token.len() - 3], Some("i16"))
    } else {
        (token, None)
    };

    // Determine if hex or decimal
    let is_hex = num_str.starts_with("0x") || num_str.starts_with("0X");

    if is_hex {
        let hex_str = &num_str[2..];

        // Determine default type based on length if no suffix
        let default_type = if hex_str.len() == 4 { "i16" } else { "u8" };
        let actual_type = suffix.unwrap_or(default_type);

        match actual_type {
            "u8" => {
                let value = u8::from_str_radix(hex_str, 16)
                    .unwrap_or_else(|_| panic!("Failed to parse hex u8: {}", num_str));
                vec![value]
            }
            "u16" => {
                let value = u16::from_str_radix(hex_str, 16)
                    .unwrap_or_else(|_| panic!("Failed to parse hex u16: {}", num_str));
                value.to_le_bytes().to_vec()
            }
            "i16" => {
                let value = i16::from_str_radix(hex_str, 16)
                    .unwrap_or_else(|_| panic!("Failed to parse hex i16: {}", num_str));
                value.to_le_bytes().to_vec()
            }
            _ => panic!("Unknown suffix: {}", actual_type),
        }
    } else {
        // Decimal parsing
        let actual_type = suffix.unwrap_or("u8");

        match actual_type {
            "u8" => {
                let value: u8 = num_str
                    .parse()
                    .unwrap_or_else(|_| panic!("Failed to parse decimal u8: {}", num_str));
                vec![value]
            }
            "u16" => {
                let value: u16 = num_str
                    .parse()
                    .unwrap_or_else(|_| panic!("Failed to parse decimal u16: {}", num_str));
                value.to_le_bytes().to_vec()
            }
            "i16" => {
                let value: i16 = num_str
                    .parse()
                    .unwrap_or_else(|_| panic!("Failed to parse decimal i16: {}", num_str));
                value.to_le_bytes().to_vec()
            }
            _ => panic!("Unknown suffix: {}", actual_type),
        }
    }
}

fn num_line_to_vec(line: &str) -> Vec<u8> {
    let mut result: Vec<u8> = Vec::new();
    let tokens = line.split_whitespace();
    for token in tokens {
        result.extend_from_slice(&parse_number(token));
    }
    result
}

fn opcode_by_name<S: Sync>(name: &str) -> Option<u8> {
    let opcodes = VM::<0, S, NoVmDebug>::opcode_names();
    for (code, op_name) in opcodes.iter() {
        if *op_name == name {
            return Some(*code);
        }
    }
    None
}

fn parse_op_args(args: &str) -> Vec<u8> {
    let mut result: Vec<u8> = Vec::new();

    // Split by comma and process each argument
    for arg in args.split(',') {
        let arg = arg.trim();
        if arg.is_empty() {
            continue;
        }

        result.extend_from_slice(&parse_number(arg));
    }

    result
}
