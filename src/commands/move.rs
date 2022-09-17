use fs_err as fs;

use clap::Parser;

use eyre::Result;

mod models;

use self::models::{Move, MoveValueParser};

#[derive(Debug, Parser)]
pub struct MoveArgs {
    /// Multiple paths in the format <source>::<destination> that should be moved.
    ///
    /// For example: refactor move /home/test.txt::/home/new/test.md
    #[clap(value_parser = MoveValueParser::new())]
    moves: Vec<Move>,
}

pub fn r#move(args: MoveArgs) -> Result<()> {
    let mut moves = args.moves;
    moves.sort_by_key(|v| v.source.clone());
    moves.reverse();
    for mv in moves {
        move_file_or_folder(&mv)?;
    }
    Ok(())
}

fn move_file_or_folder(mv: &Move) -> Result<()> {
    println!(
        "Move {} to {}",
        &mv.source.display(),
        &mv.destination.display()
    );
    if let Some(parent) = mv.destination.parent() {
        // Create destination parent folder if necessary
        let _ = fs::create_dir_all(parent);
    }

    fs::rename(&mv.source, &mv.destination)?;

    Ok(())
}
