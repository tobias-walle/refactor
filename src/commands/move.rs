use std::{
    env,
    path::{Path, PathBuf},
};

use clap::Parser;

use eyre::Result;

use crate::{
    models::{Move, MoveValueParser},
    utils::{move_file_or_folder, replace_path_usages},
};

#[derive(Debug, Parser)]
pub struct MoveArgs {
    /// Multiple paths in the format <source>::<destination> that should be moved.
    ///
    /// For example: refactor move /home/test.txt::/home/new/test.md
    #[clap(value_parser = MoveValueParser::new())]
    moves: Vec<Move>,

    /// Search for usages of the changed paths and try to replace them
    /// Each change needs to be confirmed in an interactive dialog
    #[clap(short, long, value_parser)]
    replace_usages: bool,
}

pub fn r#move(args: MoveArgs) -> Result<()> {
    let moves: Vec<_> = args.moves.into_iter().collect();
    let mut moves = absolute_moves(&moves)?;
    moves.sort_by_key(|v| v.source.clone());
    moves.reverse();

    if args.replace_usages {
        replace_path_usages(&moves)?;
    }

    for mv in &moves {
        move_file_or_folder(mv)?;
    }

    Ok(())
}

fn absolute_moves(moves: &[Move]) -> Result<Vec<Move>> {
    moves
        .iter()
        .map(|m| -> Result<Move> {
            Ok(Move {
                source: absolute_path(&m.source)?,
                destination: absolute_path(&m.destination)?,
            })
        })
        .collect()
}

fn absolute_path(relative: &Path) -> Result<PathBuf> {
    let path = env::current_dir()?
        .join(&relative)
        .to_string_lossy()
        .to_string();
    let path = path_clean::clean(&path);
    Ok(path.into())
}
