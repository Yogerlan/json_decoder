use anyhow::{Context, Result, anyhow};
use clap::Parser;
use regex::Regex;
use serde::Serialize;
use serde_json::{Map, Number, Serializer, Value, ser::PrettyFormatter};
use std::{
    fs::File,
    io::{self, BufRead, BufReader},
    path::PathBuf,
};

#[derive(Parser, Debug)]
struct Args {
    /// Encoded JSON file (defaults to stdin)
    #[arg(short, long)]
    input: Option<PathBuf>,

    /// Decoded JSON file (defaults to stdout)
    #[arg(short, long)]
    output: Option<PathBuf>,
}

struct JSONDecoder {
    encoded_list: Vec<Value>,
    decoded_data: Value,
    key_index_re: Regex,
}

impl JSONDecoder {
    fn new<R: BufRead>(mut reader: R) -> Result<Self> {
        // Read the first line
        let mut line = String::new();
        reader
            .read_line(&mut line)
            .with_context(|| "Failed to read the first line")?;
        let encoded_list: Vec<Value> =
            serde_json::from_str(&line.trim()).with_context(|| "Invalid JSON array")?;
        let decoded_data: Value = Value::Null;

        // Regular expression to match object indexes keys
        let key_index_re = Regex::new(r"^_(\d+)$").with_context(|| "Failed to compile regex")?;

        let mut decoder = JSONDecoder {
            encoded_list,
            decoded_data,
            key_index_re,
        };

        // Regular expression to match extra lines keys
        let p_index_re = Regex::new(r"^P(\d+)$").with_context(|| "Failed to compile regex")?;

        // Read extra lines
        loop {
            line.clear();
            reader
                .read_line(&mut line)
                .with_context(|| "Failed to read the extra line")?;

            if line.trim().is_empty() {
                break;
            }

            let (p_index, p_encoded_str) = line
                .split_once(":")
                .with_context(|| "Invalid extra line format")?;

            // Ensure the P-index is valid
            let index = decoder.decode_index(
                p_index_re
                    .captures(p_index.trim())
                    .with_context(|| "Invalid P-index format")?
                    .get(1)
                    .with_context(|| "Invalid P-index format")?
                    .as_str()
                    .parse::<i64>()
                    .ok(),
            )?;

            // Update the index in the corresponding array
            let len = decoder.encoded_list.len();
            let value = &mut decoder.encoded_list[index];
            let arr = value
                .as_array_mut()
                .with_context(|| "Invalid array format")?;

            if arr.len() != 2 {
                return Err(anyhow!("Array length is not 2"));
            }

            arr[1] = Value::Number(Number::from(len as u64));

            // Extend encoded_list with the parsed extra line
            let mut encoded_extra: Vec<Value> =
                serde_json::from_str(p_encoded_str.trim()).with_context(|| "Invalid JSON array")?;
            decoder.encoded_list.append(&mut encoded_extra);
        }

        decoder.decoded_data = decoder.decode_fragment(&decoder.encoded_list[0])?;

        Ok(decoder)
    }

    fn decode_fragment(&self, fragment: &Value) -> Result<Value> {
        match fragment {
            Value::Array(arr) => self.decode_array(arr),
            Value::Object(obj) => self.decode_object(obj),
            v => Ok(v.clone()),
        }
    }

    fn decode_index(&self, index: Option<i64>) -> Result<usize> {
        let r = match index {
            Some(i) if i >= 0 => {
                let u = i as usize;

                match u < self.encoded_list.len() {
                    true => u,
                    false => return Err(anyhow!("Index out of bounds")),
                }
            }
            Some(i) => {
                let u = i.abs() as usize;

                match u <= self.encoded_list.len() {
                    true => self.encoded_list.len() - u,
                    false => return Err(anyhow!("Index out of bounds")),
                }
            }
            None => return Err(anyhow!("Invalid number format")),
        };

        Ok(r)
    }

    fn decode_array(&self, arr: &[Value]) -> Result<Value> {
        let mut result = Vec::<Value>::new();

        for item in arr {
            match item {
                Value::Number(n) => {
                    let index = self.decode_index(n.as_i64())?;
                    result.push(self.decode_fragment(&self.encoded_list[index])?)
                }
                Value::String(s) if s == "P" => {
                    let index = self.decode_index(
                        arr.get(1)
                            .with_context(|| "Missing index in array")?
                            .as_i64(),
                    )?;

                    return self.decode_fragment(&self.encoded_list[index]);
                }
                f => result.push(self.decode_fragment(f)?),
            };
        }

        Ok(Value::Array(result))
    }

    fn decode_object(&self, obj: &Map<String, Value>) -> Result<Value> {
        let mut result = Map::<String, Value>::new();

        for (key, value) in obj {
            // Ensure the K-index is valid
            let mut index = self.decode_index(
                self.key_index_re
                    .captures(key)
                    .with_context(|| "Invalid K-index format")?
                    .get(1)
                    .with_context(|| "Invalid K-index format")?
                    .as_str()
                    .parse::<i64>()
                    .ok(),
            )?;

            let obj_key = String::from(
                self.encoded_list[index]
                    .as_str()
                    .with_context(|| "Invalid string format")?,
            );
            index = self.decode_index(value.as_i64())?;
            let obj_value = self.decode_fragment(&self.encoded_list[index])?;
            result.insert(obj_key, obj_value);
        }

        Ok(Value::Object(result))
    }

    fn decoded_data(&self) -> &Value {
        &self.decoded_data
    }
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Input: file or stdin
    let decoder = match args.input {
        Some(input_path) => {
            let f = File::open(input_path).with_context(|| "Failed to open input file")?;
            JSONDecoder::new(BufReader::new(f))?
        }
        None => JSONDecoder::new(io::stdin().lock())?,
    };

    // Output: file or stdin
    let formatter = PrettyFormatter::with_indent(b"    ");
    match args.output {
        Some(output_path) => {
            let f = File::create(output_path).with_context(|| "Failed to create output file")?;
            let mut ser = Serializer::with_formatter(f, formatter);
            decoder
                .decoded_data()
                .serialize(&mut ser)
                .with_context(|| "Failed to write JSON data")?
        }
        None => {
            let mut ser = Serializer::with_formatter(io::stdout().lock(), formatter);
            decoder
                .decoded_data()
                .serialize(&mut ser)
                .with_context(|| "Failed to write JSON data")?
        }
    }

    Ok(())
}
