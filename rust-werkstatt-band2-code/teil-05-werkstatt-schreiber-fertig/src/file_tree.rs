use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub(crate) struct DateiEintrag {
    pub(crate) pfad: PathBuf,
    pub(crate) name: String,
}

pub(crate) fn dateien_laden(ordner: &Path) -> Vec<DateiEintrag> {
    let mut eintraege = Vec::new();

    if let Ok(leser) = fs::read_dir(ordner) {
        for eintrag in leser.flatten() {
            let pfad = eintrag.path();
            if pfad.is_file() {
                if let Some(ext) = pfad.extension() {
                    if ext == "md" || ext == "txt" {
                        let name = pfad
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("Unbekannt")
                            .to_string();

                        eintraege.push(DateiEintrag { pfad, name });
                    }
                }
            }
        }
    }

    eintraege.sort_by(|a, b| {
        a.name.to_lowercase().cmp(&b.name.to_lowercase())
    });
    eintraege
}
