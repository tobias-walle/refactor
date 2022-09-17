use fs_err as fs;

use crate::models::Move;
use eyre::Result;

pub fn move_file_or_folder(mv: &Move) -> Result<()> {
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
