
use clap::Parser;
use std::path::Path;
use serde::Deserialize;
use serde_json;
use std::fs::{read_to_string, File};
use std::io::{BufRead, BufReader, Read};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long)]
    file: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SettingsJson {
    file_path: String,
    target_words: TargetWords
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TargetWords {
    words: Vec<String>,
    reg_ex: bool
}


fn main() {
    let args = Args::parse();

    println!("Hello {}!", args.file);

    let path = Path::new(&args.file);
    if !path.is_file() {
        println!("This is not a file");
        return;
    }
    let mut f = File::open(path).expect("file not found");

    let mut contents = String::new();
    f.read_to_string(&mut contents)
        // ファイルの読み込み中に問題がありました
        .expect("something went wrong reading the file");
    println!("With text:\n{}", contents);

    let setting_json_data: SettingsJson = serde_json::from_str(&contents).expect("JSON was not well-formatted");
    println!("{:?}", setting_json_data);

    // target file check
    let path = Path::new(&setting_json_data.file_path);
    let display = path.display();
    if !path.is_file() {
        println!("This is not a file");
        return;
    }
    let lines = read_lines(&path.display().to_string());
    let mut hitData : Vec<String> = Vec::new();
    for line in lines {
        for targetWord in setting_json_data.target_words.words.iter() {
            if line.contains(targetWord) {
                println!("{}",line);
            } else {
                // no hit
            }
        }
    }

    // read 1line on file and search TargetWords
}

fn read_lines(filename: &str) -> Vec<String> {
    read_to_string(filename)
        .unwrap()  // panic on possible file-reading errors
        .lines()  // split the string into an iterator of string slices
        .map(String::from)  // make each slice into a string
        .collect()  // gather them together into a vector
}