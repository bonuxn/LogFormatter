
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
use encoding_rs_io::DecodeReaderBytes;

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

    let mut file_content_lines: Option<Vec<String>> = None; // Option<Vec<String>> で初期化

    let result_lines = read_lines(&path.display().to_string(),None);

    match result_lines {
        Ok(lines) => {
            // lines はこのスコープ内でのみ有効
            // 外側の変数に設定するには、明示的に代入する
            file_content_lines = Some(lines);
            println!("ファイル読み込み成功（matchの結果を束縛）");
        }
        Err(e) => {
            eprintln!("ファイル読み込みエラー（matchの結果を束縛）: {}", e);
            return;
        }
    }
    let mut hit_data : Vec<Csv_data> = vec![];
    let target_words : Vec<String> = setting_json_data.target_words.words;
    let date_format : String = setting_json_data.date_format;
    let time_format : String = setting_json_data.time_format;

    println!("dateFormat={},timeFormat={}",date_format,time_format);

    hit_data.push(Csv_data {date: "date".to_string(),time: "time".to_string(),data : "data".to_string()});

    // search target word -> input hitData
    let regex_date = Regex::new(&date_format).expect("Invalid date format regex");
    let regex_time = Regex::new(&time_format).expect("Invalid date format regex");

    for line in file_content_lines.unwrap().iter()    {
        for target_word in target_words.iter() {
            if line.contains(target_word) {
                if regex_date.is_match(&line) && regex_time.is_match(&line) {
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

/// ファイルのエンコーディングを UTF-8 または Shift-JIS のどちらかであると推測します。
/// 優先順位は UTF-8 BOM > UTF-8 (BOMなし) > Shift-JIS です。
///
/// # 引数
/// * `path` - 読み込むファイルのパス
///
/// # 戻り値
/// 推測されたエンコーディング (Encodingオブジェクト) を返します。
/// どちらにも当てはまらない、または判断が困難な場合は、デフォルトで UTF-8 を返します。
pub fn guess_encoding_utf8_sjis_priority(path: &str) -> io::Result<&'static Encoding> {
    let mut file = File::open(path)?;
    let mut contents = Vec::new();
    file.read_to_end(&mut contents)?;

    // 2. UTF-8 としての有効性チェック
    // decode() は BOM を考慮しますが、ここでは BOM がないと分かっているので、
    // decode_without_bom_handling() を使って純粋なUTF-8適合性をテストします。
    // `had_errors` は、不正なバイトシーケンスがあった場合に true になります。
    let (cow, utf8_had_errors) = UTF_8.decode_without_bom_handling(&contents);

    // 3. Shift-JIS としての有効性チェック
    let (cow, sjis_had_errors) = SHIFT_JIS.decode_without_bom_handling(&contents);

    println!("UTF-8 エラー: {}, Shift-JIS エラー: {}", utf8_had_errors, sjis_had_errors);

    // 判断ロジック
    if !utf8_had_errors && !sjis_had_errors {
        // 両方ともエラーなくデコードできる場合 (例: ASCIIのみのファイル)
        // 一般的にはUTF-8を優先します。
        println!("両方エラーなし。UTF-8を優先します。");
        Ok(UTF_8)
    } else if !utf8_had_errors {
        // UTF-8のみエラーなし
        println!("UTF-8と推測 (エラーなし)。");
        Ok(UTF_8)
    } else if !sjis_had_errors {
        // Shift-JISのみエラーなし
        println!("Shift-JISと推測 (エラーなし)。");
        Ok(SHIFT_JIS)
    } else {
        // 両方ともエラーがある場合
        // この場合、どちらのエンコーディングでもない可能性が高いですが、
        // 強いて選ぶならUTF-8を返します（またはエラーを返す）。
        // ここではUTF-8を返しますが、ユーザーにエンコーディング選択を促すのがより堅牢です。
        println!("どちらもエラーあり。UTF-8をフォールバックします。");
        Ok(UTF_8)
    }
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

fn read_lines(filepath: &str,encoding_override: Option<&'static Encoding>) -> Result<Vec<String>,io::Error> {
    let file = File::open(filepath)?;
    let mut reader = BufReader::new(file);

    let mut contents = Vec::new();
    reader.read_to_end(&mut contents)?; // ファイル全体をバイト列として読み込む

    let encoding = guess_encoding_utf8_sjis_priority(filepath)?;
    let (cow_str, _, had_errors) = encoding.decode(&contents);

    // デコードされた文字列を行に分割
    let lines: Vec<String> = cow_str
        .lines() // 改行コードで分割
        .map(|s| s.to_string())
        .collect();

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