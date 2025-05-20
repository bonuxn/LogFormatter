
use clap::Parser;
use std::path::Path;
use serde::Deserialize;
use serde_json;
use std::fs::{create_dir_all,read_to_string, File};
use std::io::{BufWriter, Write, BufRead, Read, BufReader};
use std::collections::HashMap;
use std::{env, fs, io};
use regex::Regex;
use chrono::prelude::{DateTime, Local};
use csv::{WriterBuilder, Writer};
use encoding_rs::{Encoding, SHIFT_JIS, UTF_8};

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

#[derive(Debug)]
struct Csv_data {
    date: String,
    time: String,
    data: String,
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
    let mut hit_data : Vec<Csv_data> = vec![];
    let mut hit_data_no :u64 = 1;
    let target_words : Vec<String> = setting_json_data.target_words.words;
    let date_format : String = setting_json_data.date_format;
    let time_format : String = setting_json_data.time_format;

    println!("dateFormat={},timeFormat={}",date_format,time_format);

    hit_data.push(Csv_data {date: "date".to_string(),time: "time".to_string(),data : "data".to_string()});

    // search target word -> input hitData
    let regex_date = Regex::new(&date_format).expect("Invalid date format regex");
    let regex_time = Regex::new(&time_format).expect("Invalid date format regex");

    for line in lines.unwrap().lines() {
        println!("hit: {}", line);
        for target_word in target_words.iter() {
            if line.contains(target_word) {
                if regex_date.is_match(&line) && regex_time.is_match(&line) {
                    hit_data_no += 1;
                    let date_caps = regex_date.captures(&line).unwrap();
                    let time_caps = regex_time.captures(&line).unwrap();
                    let date_data = date_caps.get(0).unwrap().as_str();
                    let time_data = time_caps.get(0).unwrap().as_str();
                    hit_data.push(Csv_data{ date: date_data.to_string(), time: time_data.to_string(), data: line.to_string() });
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

fn read_lines(filepath: &str) -> Result<Vec<String>,io::Error> {
    let file = File::open(filepath)?;
    let mut reader = BufReader::new(file);

    // 1. BufReaderのバッファから最初の数バイトを取得してエンコーディングを判別
    let (first_bytes, _len) = {
        let buffer = reader.fill_buf()?; // バッファを埋める（実際の読み込みはここで発生）
        // エンコーディング判別には最初の数バイトで十分なことが多い
        // ここでは最大1024バイト、またはバッファに存在する全てのバイトを使用
        let sample_len = buffer.len().min(1024);
        (buffer[..sample_len].to_vec(), sample_len)
    };

    let detected_encoding: &'static Encoding;

    // UTF-8としてデコードを試みる
    let (_, _, had_errors_utf8) = UTF_8.decode(&first_bytes);

    if !had_errors_utf8 {
        println!("INFO: File '{}' detected as UTF-8 based on initial bytes.", filepath);
        detected_encoding = UTF_8;
    } else {
        println!("INFO: UTF-8 decoding for initial bytes of '{}' contained errors. Attempting Shift-JIS...", filepath);
        let (_, _, had_errors_sjis) = SHIFT_JIS.decode(&first_bytes);

        if !had_errors_sjis {
            println!("INFO: File '{}' successfully decoded as Shift-JIS based on initial bytes.", filepath);
            detected_encoding = SHIFT_JIS;
        } else {
            eprintln!("WARNING: Both UTF-8 and Shift-JIS decoding for initial bytes of '{}' contained errors. Defaulting to UTF-8 for line-by-line reading.", filepath);
            detected_encoding = UTF_8;
        }
    }

    // 2. 判別したエンコーディングで、BufReaderを使って行ごとに読み込む
    let mut lines = Vec::new();
    let mut decoder = detected_encoding.new_decoder(); // 新しいデコーダーを生成
    let mut line_buffer = Vec::new(); // バイト列の行を格納するバッファ

    // reader.bytes() を使って、残りのファイル内容をバイト単位で処理
    // ここで reader は最初の fill_buf() で読み込んだ部分から処理を再開します。
    for read_line_result in reader.bytes() {
        let byte = read_line_result?;
        line_buffer.push(byte);

        // 改行コード (LF: 0x0A, CR: 0x0D) を検出
        if byte == b'\n' || byte == b'\r' {
            let (cow, _, _) = decoder.decode_to_string(&line_buffer);
            lines.push(cow.into_owned());
            line_buffer.clear();
            decoder.reset();
        }
    }

    // ファイルの終端でバッファに残っているデータがあれば処理 (最後の行など)
    if !line_buffer.is_empty() {
        let (cow, _, _) = decoder.decode_to_string(&line_buffer);
        lines.push(cow.into_owned());
    }

    Ok(lines)

}

fn write_csv_file(records: Vec<Csv_data>) -> Result<(), String> {
    // 出力先ディレクトリを設定
    let output_dir = env::current_dir().unwrap();
    create_directory(&output_dir)?;

    let filename : String = get_date_time_string();

    // ファイルパスを生成し、ファイルを作成
    let file_path = output_dir.join(filename + ".csv");

    // CSVファイルを開く (タブ区切りで設定)
    let mut wtr = WriterBuilder::new()
        .delimiter(b'\t') // デリミタをタブに設定
        .from_writer(File::create(file_path).expect("File can not be created"));

    // データを書き込む
    for record in records {
        wtr.write_record(
            &[record.date.to_string(), record.time.to_string(), record.data.to_string()])
            .expect("Write record");
    }

    wtr.flush().expect("File can not be flushed");
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

fn write_csv_content(writer: &mut BufWriter<File>, data: &HashMap<u64, String>) -> Result<(), String> {
    for (_, value) in data {
        writeln!(writer, "{}", value).map_err(|e| format!("Failed to write line: {}", e))?;
    }
    Ok(())
}

fn get_date_time_string() -> String {
    let mut yyyymmddhhmmss : String = "99999999999999".to_string();
    let local: DateTime<Local> = Local::now();
    yyyymmddhhmmss = local.format("%Y%m%d_%H%M%S").to_string();

    println!("Now:\n local: {:?}", local.to_string());
    println!("local: {:?}", yyyymmddhhmmss);

    yyyymmddhhmmss
}