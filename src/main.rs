
use clap::Parser;
use std::path::Path;
use serde::Deserialize;
use serde_json;
use std::fs::{create_dir_all,read_to_string, File};
use std::io::{self,BufWriter, Write,BufRead, BufReader, Read};
use std::collections::HashMap;
use std::env;
use regex::Regex;

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
    target_words: TargetWords,
    date_format: String,
    time_format: String
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TargetWords {
    words: Vec<String>,
    reg_ex: bool
}
const BOM: &[u8; 3] = &[0xEF, 0xBB, 0xBF];

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
    let mut hit_data = HashMap::new();
    let mut hit_data_no :u64 = 1;
    let target_words : Vec<String> = setting_json_data.target_words.words;
    let date_format : String = setting_json_data.date_format;
    let time_format : String = setting_json_data.time_format;

    println!("dateFormat={},timeFormat={}",date_format,time_format);

    hit_data.insert(hit_data_no,concat!("date","\t", "time","\t","data"));

    // search target word -> input hitData
    let regex_date = Regex::new(&date_format).expect("Invalid date format regex");
    let regex_time = Regex::new(&time_format).expect("Invalid date format regex");

    for line in lines {
        for target_word in target_words.iter() {
            if line.contains(target_word) {
                if regex_date.is_match(&line) && regex_time.is_match(&line) {
                    hit_data_no += 1;
                    let date_caps = regex_date.captures(&line).unwrap();
                    let time_caps = regex_time.captures(&line).unwrap();
                    let date_data = date_caps.get(0).unwrap().as_str();
                    let time_data = time_caps.get(0).unwrap().as_str();
                    hit_data.insert(hit_data_no,&format!("{} {} {} {} {}", &date_data, "\t", &time_data, "\t", &line));
                    println!("{} : {} : {}",date_data,time_data,line);
                }
            } else {
                // no hit
            }
        }
    }
    write_csv_file(hit_data).expect("TODO: panic message");
    // read 1line on file and search TargetWords
}


fn extract_date_from_string(input: &str, date_format: &str) -> Option<String> {

    // Create a regex object from the date_format string
    let regex = Regex::new(date_format).expect("Invalid date format regex");

    // Find the first match in the input string
    if let Some(captures) = regex.captures(input) {
        // Return the matching date substring
        if let Some(matched) = captures.get(0) {
            return Some(matched.as_str().to_string());
        }
    }

    None
}

fn read_lines(filename: &str) -> Vec<String> {
    read_to_string(filename)
        .unwrap()  // panic on possible file-reading errors
        .lines()  // split the string into an iterator of string slices
        .map(String::from)  // make each slice into a string
        .collect()  // gather them together into a vector
}

fn write_csv_file(data: HashMap<u64, &str>) -> Result<(), String> {
    // 出力先ディレクトリを設定
    let output_dir = env::current_dir().unwrap();
    create_directory(&output_dir)?;

    // ファイルパスを生成し、ファイルを作成
    let file_path = output_dir.join("fruits.csv");
    let file = File::create(&file_path).map_err(|e| format!("Failed to create file: {}", e))?;

    // ファイル書き込み処理
    let mut writer = BufWriter::new(file);
    write_bom(&mut writer)?;
    write_csv_content(&mut writer, &data)?;

    Ok(())
}
fn create_directory(path: &Path) -> Result<(), String> {
    create_dir_all(path).map_err(|e| format!("Failed to create directory: {}", e))
}

fn write_bom(writer: &mut BufWriter<File>) -> Result<(), String> {
    writer
        .write_all(BOM)
        .map_err(|e| format!("Failed to write BOM: {}", e))
}

fn write_csv_content(writer: &mut BufWriter<File>, data: &HashMap<u64, &str>) -> Result<(), String> {
    for (_, value) in data {
        writeln!(writer, "{}", value).map_err(|e| format!("Failed to write line: {}", e))?;
    }
    Ok(())
}