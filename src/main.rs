use regex::Regex;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

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
            let indent = &caps[1];
            let original = &caps[2];
            let struct_name = &caps[3];
            format!(
                "{}/// WIP_{}_struct_description{}{}",
                indent, struct_name, indent, original
            )
        })
        .to_string();

    // enum のコメントを追加
    new_content = enum_re
        .replace_all(&new_content, |caps: &regex::Captures| {
            let indent = &caps[1];
            let original = &caps[2];
            let enum_name = &caps[3];
            format!(
                "{}/// WIP_{}_enum_description\n{}{}",
                indent, enum_name, indent, original
            )
        })
        .to_string();

    // trait のコメントを追加
    new_content = trait_re
        .replace_all(&new_content, |caps: &regex::Captures| {
            let indent = &caps[1];
            let original = &caps[2];
            let trait_name = &caps[3];
            format!(
                "{}/// WIP_{}_trait_description{}{}",
                indent, trait_name, indent, original
            )
        })
        .to_string();

    // 関数のコメントを追加
    new_content = function_re
        .replace_all(&new_content, |caps: &regex::Captures| {
            let indent = &caps[1];
            let function_name = &caps[2];
            format!(
                "{}/// WIP_{}_function_description{}pub fn {}(",
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

// ディレクトリ内のすべてのRustファイルにコメントを追加
fn process_directory(path: &Path) -> io::Result<()> {
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            process_directory(&path)?;
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            let file_content = fs::read_to_string(&path)?;
            let new_content = add_comments_to_struct_enum_trait(&file_content);
            let mut file = fs::File::create(&path)?;
            file.write_all(new_content.as_bytes())?;
        }
    }
    Ok(())
}

fn main() -> io::Result<()> {
    let src_path = Path::new("src");
    process_directory(src_path)
}
