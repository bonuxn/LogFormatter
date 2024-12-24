use std::error::Error;
use clap::Parser;
use std::path::Path;
use serde::Deserialize;
use serde_json;
use std::fs::File;
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
    let f = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", display,
                           why.description()),
        Ok(file) => file,
    };
    let reader = BufReader::new(f);
    for line in reader.lines() {
        println!("{}", line.unwrap());
    }
}