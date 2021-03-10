use quick_xml::{events::Event, Reader};
use rayon::prelude::*;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use structopt::StructOpt;

#[derive(StructOpt)]
struct Args {
    files: Vec<PathBuf>,
}

fn process(file: &Path) -> Result<Vec<(String, String)>, quick_xml::Error> {
    let mut out = Vec::with_capacity(1000);
    let mut buf = Vec::with_capacity(1024);
    let mut reader = Reader::from_file(file)?;
    let mut word = String::new();
    let mut hanja = String::new();

    loop {
        match reader.read_event(&mut buf)? {
            Event::Start(ref e) => {
                match e.name() {
                    b"word" => {
                        // drop text
                        reader.read_event(&mut buf)?;
                        if let Event::CData(ref data) = reader.read_event(&mut buf)? {
                            word = data.unescape_and_decode(&reader)?;
                        }
                    }
                    b"original_language" => {
                        // drop text
                        reader.read_event(&mut buf)?;
                        if let Event::CData(ref data) = reader.read_event(&mut buf)? {
                            hanja = data.unescape_and_decode(&reader)?;
                        }
                    }
                    b"language_type" => {
                        reader.read_event(&mut buf)?;
                        if let Event::CData(ref data) = reader.read_event(&mut buf)? {
                            if data.escaped() == "한자".as_bytes() {
                                out.push((std::mem::take(&mut word), std::mem::take(&mut hanja)));
                            }
                        }
                    }
                    _ => {}
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }

    Ok(out)
}

fn main() {
    let args = Args::from_args();

    let dict: BTreeMap<String, String> = args
        .files
        .into_par_iter()
        .flat_map(|file| process(&file).unwrap_or_default())
        .collect();

    println!("{:#?}", dict);
}
