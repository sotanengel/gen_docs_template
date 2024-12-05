use regex::Regex;
use std::env;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

/// ファイル内の `struct`、`enum`、`trait` および `pub fn` 宣言にコメントを追加する関数
fn add_comments_to_struct_enum_trait(file_content: &str) -> String {
    let struct_pattern = r"(?m)^([\s]*)((?:#\[.*\]\s*)*pub\s+struct\s+(\w+))";
    let enum_pattern = r"(?m)^([\s]*)((?:#\[.*\]\s*)*pub\s+enum\s+(\w+))";
    let trait_pattern = r"(?m)^([\s]*)((?:#\[.*\]\s*)*pub\s+trait\s+(\w+))";
    let function_pattern = r"(?m)^([\s]*)pub\s+fn\s+(\w+)\s*\(";

    let struct_re = Regex::new(struct_pattern).unwrap();
    let enum_re = Regex::new(enum_pattern).unwrap();
    let trait_re = Regex::new(trait_pattern).unwrap();
    let function_re = Regex::new(function_pattern).unwrap();
    let field_re = Regex::new(r"(\s+pub\s+(\w+):)").unwrap();
    let variant_re = Regex::new(r"(\s+(\w+),)").unwrap();

    let mut new_content = file_content.to_string();

    // struct のコメントを追加
    new_content = struct_re
        .replace_all(&new_content, |caps: &regex::Captures| {
            let indent = &caps[1].trim(); // インデントから改行を削除
            let original = &caps[2];
            let struct_name = &caps[3];
            format!(
                "{}\n/// WIP_{}_struct_description\n{}{}",
                indent, struct_name, indent, original
            )
        })
        .to_string();

    // enum のコメントを追加
    new_content = enum_re
        .replace_all(&new_content, |caps: &regex::Captures| {
            let indent = &caps[1].trim(); // インデントから改行を削除
            let original = &caps[2];
            let enum_name = &caps[3];
            format!(
                "{}\n/// WIP_{}_enum_description\n{}{}",
                indent, enum_name, indent, original
            )
        })
        .to_string();

    // trait のコメントを追加
    new_content = trait_re
        .replace_all(&new_content, |caps: &regex::Captures| {
            let indent = &caps[1].trim(); // インデントから改行を削除
            let original = &caps[2];
            let trait_name = &caps[3];
            format!(
                "{}\n/// WIP_{}_trait_description\n{}{}",
                indent, trait_name, indent, original
            )
        })
        .to_string();

    // 関数のコメントを追加
    new_content = function_re
        .replace_all(&new_content, |caps: &regex::Captures| {
            let indent = &caps[1].trim(); // インデントから改行を削除
            let function_name = &caps[2];
            format!(
                "{}\n/// WIP_{}_function_description\n{}pub fn {}(",
                indent, function_name, indent, function_name
            )
        })
        .to_string();

    // 構造体フィールド、enum バリアントのコメント追加
    let mut final_content = String::new();
    let mut in_struct = false;
    let mut in_enum = false;
    let mut brace_count = 0;

    for line in new_content.lines() {
        if line.contains("pub struct") {
            in_struct = true;
            brace_count = 0;
        } else if line.contains("pub enum") {
            in_enum = true;
            brace_count = 0;
        }

        if in_struct || in_enum {
            brace_count += line.matches('{').count() as i32;
            brace_count -= line.matches('}').count() as i32;

            if brace_count == 0 && line.contains('}') {
                in_struct = false;
                in_enum = false;
            }
        }

        if in_struct {
            if let Some(caps) = field_re.captures(line) {
                let field_name = &caps[2];
                final_content.push_str(&format!("    /// WIP_{}_field_description\n", field_name));
            }
        } else if in_enum {
            if let Some(caps) = variant_re.captures(line) {
                let variant_name = &caps[2];
                final_content.push_str(&format!(
                    "    /// WIP_{}_variant_description\n",
                    variant_name
                ));
            }
        }

        final_content.push_str(line);
        final_content.push('\n');
    }

    final_content
}

/// 履歴ファイルから既存の処理済みファイルのリストを取得する関数
fn get_processed_files() -> io::Result<Vec<String>> {
    let history_file = Path::new(".gen_doc_his");

    // ファイルが存在しない場合、空のリストを返す
    if !history_file.exists() {
        return Ok(Vec::new());
    }

    // 履歴ファイルの内容を1行ずつ読み込む
    let content = fs::read_to_string(history_file)?;
    let lines: Vec<String> = content.lines().map(|line| line.to_string()).collect();

    Ok(lines)
}

// ディレクトリ内のすべてのRustファイルにコメントを追加し、履歴ファイルにパスを記録
fn process_directory(path: &Path, processed_files: &[String]) -> io::Result<()> {
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            process_directory(&path, processed_files)?;
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            let path_str = path.display().to_string();

            // 処理済みファイルかどうかを確認
            if processed_files.contains(&path_str) {
                println!("Skipping already comments added file: {}", path_str);
                continue;
            }

            // ファイル内容を読み込み、コメントを追加
            let file_content = fs::read_to_string(&path)?;
            let new_content = add_comments_to_struct_enum_trait(&file_content);

            // ファイルに書き込み
            let mut file = fs::File::create(&path)?;
            file.write_all(new_content.as_bytes())?;

            // 処理したファイルを履歴ファイルに追加
            append_to_history(&path)?;
        }
    }
    Ok(())
}

/// ファイルパスを `.gen_doc_his` に追記する関数
fn append_to_history(file_path: &Path) -> io::Result<()> {
    let history_file = Path::new(".gen_doc_his");
    let is_new_file = !history_file.exists();

    // ファイルが存在する場合は追記モード、存在しない場合は新規作成
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(history_file)?;

    // 新規作成時のみ `.gitignore` に `.gen_doc_his` を追加
    if is_new_file {
        let gitignore_file = Path::new(".gitignore");
        if gitignore_file.exists() {
            // `.gitignore` の内容を確認
            let content = fs::read_to_string(gitignore_file)?;
            if !content.lines().any(|line| line.trim() == ".gen_doc_his") {
                // `.gen_doc_his` がまだ存在しない場合のみ追加
                let mut gitignore = OpenOptions::new().append(true).open(gitignore_file)?;
                writeln!(gitignore, "\n# The history file for `gen_doc_his` command.")?;
                writeln!(gitignore, "# If you want to know details, please access 'https://crates.io/crates/gen_docs_template'.")?;
                writeln!(gitignore, ".gen_doc_his")?;
            }
        }
    }

    // ファイルパスを追記
    writeln!(file, "{}", file_path.display())?;
    Ok(())
}

fn remove_history_file() -> io::Result<()> {
    let history_file = Path::new(".gen_doc_his");
    if history_file.exists() {
        fs::remove_file(history_file)?;
        println!("Removed history file: {}", history_file.display());
    }
    Ok(())
}

fn main() -> io::Result<()> {
    // コマンドライン引数を取得
    let args: Vec<String> = env::args().collect();

    // `hard`オプションの処理
    if args.iter().any(|arg| arg == "hard") {
        remove_history_file()?;
    }

    let args: Vec<String> = env::args().collect();
    let target_path: PathBuf = if args.len() > 2 && args[1] == "-path" {
        PathBuf::from(&args[2])
    } else {
        PathBuf::from("src")
    };

    // 履歴ファイルから処理済みファイルリストを取得
    let processed_files = get_processed_files()?;

    // 指定されたディレクトリに対して処理を実行
    if target_path.exists() && target_path.is_dir() {
        process_directory(&target_path, &processed_files)?;
    } else {
        eprintln!("Error Folder not Found.. : {:?}", target_path);
        std::process::exit(1);
    }

    Ok(())
}
