use crossterm::{
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{Clear, ClearType},
    ExecutableCommand,
};
use fs_err as fs;
use pathdiff::diff_paths;
use std::{
    env,
    io::{stdin, stdout},
    path::{Path, PathBuf},
    process,
};

use crate::models::Move;
use eyre::Result;
use ignore::Walk;
use similar::{Change, ChangeTag, DiffOp, DiffTag, TextDiff};

pub fn replace_path_usages(moves: &[Move]) -> Result<()> {
    let cwd = env::current_dir()?;
    let paths: Vec<_> = Walk::new(cwd)
        .flat_map(|result| match result {
            Ok(entry) => Some(PathBuf::from(entry.path().to_string_lossy().to_string())),
            Err(err) => {
                eprintln!("ERROR: {err}");
                None
            }
        })
        .collect();

    paths.iter().for_each(|p| {
        let _ = replace_content(p, moves);
    });

    Ok(())
}

fn replace_content(path: &Path, moves: &[Move]) -> Result<()> {
    let metadata = fs::metadata(path)?;
    if !metadata.is_file() {
        return Ok(());
    }

    let original = fs::read_to_string(path)?;
    let replaced = replace_path_usages_in_file_content(path, &original, moves);

    if original == replaced {
        return Ok(());
    }

    let diff = TextDiff::from_lines(&original, &replaced);

    let lines_replaced: Vec<_> = replaced.split('\n').map(|s| s.to_string()).collect();
    let mut lines_result: Vec<_> = original.split('\n').map(|s| s.to_string()).collect();

    for hunk in diff.unified_diff().context_radius(10).iter_hunks() {
        clear_stdout()?;
        print_header(path)?;

        for change in hunk.iter_changes() {
            print_change(change)?;
        }

        let accepted = match ask_for_user_action()? {
            UserAction::Accept => true,
            UserAction::Decline => false,
            UserAction::Cancel => process::exit(0),
            UserAction::SkipFile => return Ok(()),
        };

        if accepted {
            accept_changes(&mut lines_result, &lines_replaced, hunk.ops());
        }
    }

    let result = lines_result.join("\n");
    fs::write(path, result)?;

    clear_stdout()?;
    Ok(())
}

fn replace_path_usages_in_file_content(path: &Path, content: &str, moves: &[Move]) -> String {
    let mut content = content.to_string();
    for mv in moves {
        if let Some(replacement) = replace_direct_path_usages(path, &content, mv) {
            content = replacement;
        };
        if let Some(replacement) = replace_indirect_path_usages(&content, mv) {
            content = replacement;
        };
    }
    content
}

fn replace_direct_path_usages(path: &Path, content: &str, mv: &Move) -> Option<String> {
    let parent_folder = &path.parent()?;
    let relative_source = diff_paths(&mv.source, parent_folder)?;
    let relative_destination = diff_paths(&mv.destination, parent_folder)?;
    let content = content.replace(
        &relative_source.to_string_lossy().to_string(),
        &relative_destination.to_string_lossy(),
    );
    let content = replace_without_extension(&content, &relative_source, &relative_destination)?;
    Some(content)
}

fn replace_indirect_path_usages(content: &str, mv: &Move) -> Option<String> {
    let parent = find_common_parent(&mv.source, &mv.destination);
    let parent = parent.parent()?;
    let relative_source = diff_paths(&mv.source, &parent)?;
    let relative_destination = diff_paths(&mv.destination, &parent)?;
    let content = content.replace(
        &relative_source.to_string_lossy().to_string(),
        &relative_destination.to_string_lossy(),
    );
    let content = replace_without_extension_with_prefix(
        &content,
        &relative_source,
        &relative_destination,
        "/",
    )?;
    Some(content)
}

fn replace_without_extension(content: &str, source: &Path, destination: &Path) -> Option<String> {
    replace_without_extension_with_prefix(content, source, destination, "")
}

fn replace_without_extension_with_prefix(
    content: &str,
    source: &Path,
    destination: &Path,
    prefix: &str,
) -> Option<String> {
    let relative_source = source.with_extension("");
    let relative_destination = destination.with_extension("");
    let content = content.replace(
        &format!("{prefix}{}\"", &relative_source.to_string_lossy()),
        &format!("{prefix}{}\"", &relative_destination.to_string_lossy()),
    );
    let content = content.replace(
        &format!("{prefix}{}'", &relative_source.to_string_lossy()),
        &format!("{prefix}{}'", &relative_destination.to_string_lossy()),
    );
    Some(content)
}

fn find_common_parent(p1: &Path, p2: &Path) -> PathBuf {
    let p1: Vec<String> = p1.iter().map(|s| s.to_string_lossy().to_string()).collect();
    let p2: Vec<String> = p2.iter().map(|s| s.to_string_lossy().to_string()).collect();
    let mut results: Vec<String> = vec![];
    let mut i = 0;
    while let (Some(e), Some(b)) = (p1.get(i), p2.get(i)) {
        if e != b {
            break;
        }
        results.push(e.to_string());
        i += 1;
    }
    results.iter().collect()
}

fn clear_stdout() -> Result<()> {
    stdout().execute(Clear(ClearType::All))?;
    Ok(())
}

fn print_header(path: &Path) -> Result<()> {
    stdout()
        .execute(SetForegroundColor(Color::Magenta))?
        .execute(Print(format!("\n\n{}\n", path.to_string_lossy())))?
        .execute(ResetColor)?;
    Ok(())
}

fn print_change(change: Change<&str>) -> Result<()> {
    let text = change.to_string();

    let line = change.new_index().or_else(|| change.old_index());
    if let Some(line) = line {
        let line = line + 1;
        stdout()
            .execute(SetForegroundColor(Color::DarkGrey))?
            .execute(Print(format!("{line:>4} ")))?
            .execute(ResetColor)?;
    }

    match change.tag() {
        ChangeTag::Delete => {
            stdout()
                .execute(SetForegroundColor(Color::Red))?
                .execute(Print(text))?
                .execute(ResetColor)?;
        }
        ChangeTag::Insert => {
            stdout()
                .execute(SetForegroundColor(Color::Green))?
                .execute(Print(text))?
                .execute(ResetColor)?;
        }
        ChangeTag::Equal => {
            stdout().execute(Print(text))?;
        }
    }

    Ok(())
}

enum UserAction {
    Accept,
    Decline,
    Cancel,
    SkipFile,
}

fn ask_for_user_action() -> Result<UserAction> {
    stdout().execute(Print("\n(Y_es/n_o/C_ancel/S_kip file)> "))?;
    let mut input = String::new();
    stdin().read_line(&mut input)?;
    let action = match input.trim() {
        "y" | "Y" | "" => UserAction::Accept,
        "C" => UserAction::Cancel,
        "S" => UserAction::SkipFile,
        _ => UserAction::Decline,
    };
    Ok(action)
}

fn accept_changes(lines: &mut Vec<String>, lines_new: &[String], ops: &[DiffOp]) {
    for (tag, r1, r2) in ops.iter().map(|o| o.as_tag_tuple()).rev() {
        match tag {
            DiffTag::Delete => {
                lines.drain(r2);
            }
            DiffTag::Insert | DiffTag::Replace => {
                let to_insert = lines_new[r2].iter().map(|s| s.to_string());
                lines.splice(r1, to_insert);
            }
            DiffTag::Equal => (),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn should_replace_normal_imports() {
        let moves = vec![Move {
            source: "/src/lib/test.ts".into(),
            destination: "/src/lib/test/utils.ts".into(),
        }];
        let path = PathBuf::from("/src/main/run.ts");
        let content = r#"
        import * from "../lib/test.ts";
        
        console.log("Run");
        "#;
        let expected = r#"
        import * from "../lib/test/utils.ts";
        
        console.log("Run");
        "#;

        let replacement = replace_path_usages_in_file_content(&path, content, &moves);

        assert_eq!(expected, replacement);
    }

    #[test]
    fn should_replace_imports_without_extension() {
        let moves = vec![Move {
            source: "/src/lib/test.ts".into(),
            destination: "/src/lib/test/utils.ts".into(),
        }];
        let path = PathBuf::from("/src/main/run.ts");
        let content = r#"
        import * from "../lib/test";
        
        console.log("Run");
        "#;
        let expected = r#"
        import * from "../lib/test/utils";
        
        console.log("Run");
        "#;

        let replacement = replace_path_usages_in_file_content(&path, content, &moves);

        assert_eq!(expected, replacement);
    }

    #[test]
    fn should_replace_imports_without_extension_single_quote() {
        let moves = vec![Move {
            source: "/src/lib/test.ts".into(),
            destination: "/src/lib/test/utils.ts".into(),
        }];
        let path = PathBuf::from("/src/main/run.ts");
        let content = r#"
        import * from '../lib/test';
        
        console.log("Run");
        "#;
        let expected = r#"
        import * from '../lib/test/utils';
        
        console.log("Run");
        "#;

        let replacement = replace_path_usages_in_file_content(&path, content, &moves);

        assert_eq!(expected, replacement);
    }

    #[test]
    fn should_replace_absolute_imports() {
        let moves = vec![Move {
            source: "/src/lib/test.ts".into(),
            destination: "/src/lib/test/utils.ts".into(),
        }];
        let path = PathBuf::from("/src/main/run.ts");
        let content = r#"
        import * from "@app/lib/test";

        const path = "lib/test";
        
        console.log("Run");
        "#;
        let expected = r#"
        import * from "@app/lib/test/utils";

        const path = "lib/test";
        
        console.log("Run");
        "#;

        let replacement = replace_path_usages_in_file_content(&path, content, &moves);

        assert_eq!(expected, replacement);
    }
}
