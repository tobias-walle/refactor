

use clap::Parser;

use eyre::Result;

use crate::{
    models::{Move, MoveValueParser},
    utils::move_file_or_folder,
};

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
