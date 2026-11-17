use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Semantische Kategorie eines Eintrags für barrierefreie Farbkodierung.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kategorie {
    /// Aufräumbare Build- und Cache-Ordner (`target`, `node_modules`, `dist`, `build`).
    BuildArtefakt,
    /// Quellcode- und Konfigurationsdateien (`.rs`, `.toml`, `.md`, `.py`, ...).
    Quellcode,
    /// Bilder, Audio, Video und Schriftarten.
    Medien,
    /// Reguläre Unterordner.
    Ordner,
    /// Alle übrigen Dateien.
    Sonstiges,
}

impl Kategorie {
    /// Lesbare Bezeichnung für Text-Redundanz (Barrierefreiheit).
    pub fn bezeichnung(self) -> &'static str {
        match self {
            Kategorie::BuildArtefakt => "Build-Artefakt",
            Kategorie::Quellcode => "Quellcode",
            Kategorie::Medien => "Medien",
            Kategorie::Ordner => "Ordner",
            Kategorie::Sonstiges => "Sonstiges",
        }
    }
}

/// Ein Knoten im Dateisystem-Baum.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Knoten {
    pub name: String,
    pub pfad: PathBuf,
    pub groesse: u64,
    pub ist_ordner: bool,
    pub kategorie: Kategorie,
    pub kinder: Vec<Knoten>,
}

/// Rechteckiger Zeichenbereich in Pixeln.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bereich {
    pub x: f32,
    pub y: f32,
    pub breite: f32,
    pub hoehe: f32,
}

impl Bereich {
    pub fn enthaelt(&self, px: f32, py: f32) -> bool {
        px >= self.x
            && px <= self.x + self.breite
            && py >= self.y
            && py <= self.y + self.hoehe
    }
}

/// Eine berechnete Kachel der Treemap für die Anzeige und das Hit-Testing.
#[derive(Debug, Clone, PartialEq)]
pub struct Kachel {
    pub name: String,
    pub pfad: PathBuf,
    pub groesse: u64,
    pub anteil: f32,
    pub ist_ordner: bool,
    pub kategorie: Kategorie,
    pub bereich: Bereich,
}

/// Erkennt bekannte Build-Artefakt-Ordner anhand ihres Namens.
pub fn ist_build_artefakt_name(name: &str) -> bool {
    matches!(name, "target" | "node_modules" | ".venv" | "dist" | "build")
}

/// Ordnet eine Datei anhand ihrer Endung einer semantischen Kategorie zu.
pub fn datei_kategorie(pfad: &Path) -> Kategorie {
    let ext = pfad
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_ascii_lowercase());

    match ext.as_deref() {
        Some(
            "rs" | "toml" | "lock" | "md" | "txt" | "json" | "yaml" | "yml" | "py" | "js" | "ts"
            | "css" | "html" | "sql" | "sh",
        ) => Kategorie::Quellcode,
        Some(
            "png" | "jpg" | "jpeg" | "webp" | "svg" | "gif" | "wav" | "mp3" | "ogg" | "mp4"
            | "ttf" | "otf" | "woff2",
        ) => Kategorie::Medien,
        _ => Kategorie::Sonstiges,
    }
}

/// Formatiert Byte-Größen menschenlesbar (B, KB, MB, GB).
pub fn format_groesse(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;

    let b = bytes as f64;
    if b >= GB {
        format!("{:.2} GB", b / GB)
    } else if b >= MB {
        format!("{:.1} MB", b / MB)
    } else if b >= KB {
        format!("{:.1} KB", b / KB)
    } else {
        format!("{bytes} B")
    }
}

/// Scannt ein Verzeichnis rekursiv und sortiert Kinder nach Größe absteigend.
/// Symlinks werden übersprungen, unlesbare Einträge freundlich ignoriert.
pub fn verzeichnis_scannen(pfad: &Path) -> io::Result<Knoten> {
    let meta = fs::symlink_metadata(pfad)?;
    let name = pfad
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| pfad.display().to_string());

    if !meta.is_dir() {
        return Ok(Knoten {
            name,
            pfad: pfad.to_path_buf(),
            groesse: meta.len(),
            ist_ordner: false,
            kategorie: datei_kategorie(pfad),
            kinder: Vec::new(),
        });
    }

    let mut kinder = Vec::new();
    let mut gesamt = 0u64;

    if let Ok(eintraege) = fs::read_dir(pfad) {
        for eintrag in eintraege.flatten() {
            let kind_pfad = eintrag.path();
            let Ok(kind_meta) = fs::symlink_metadata(&kind_pfad) else {
                continue;
            };
            if kind_meta.file_type().is_symlink() {
                continue;
            }
            if let Ok(kind_knoten) = verzeichnis_scannen(&kind_pfad) {
                gesamt = gesamt.saturating_add(kind_knoten.groesse);
                if kind_knoten.groesse > 0 {
                    kinder.push(kind_knoten);
                }
            }
        }
    }

    kinder.sort_by(|a, b| b.groesse.cmp(&a.groesse).then_with(|| a.name.cmp(&b.name)));

    let kategorie = if ist_build_artefakt_name(&name) {
        Kategorie::BuildArtefakt
    } else {
        Kategorie::Ordner
    };

    Ok(Knoten {
        name,
        pfad: pfad.to_path_buf(),
        groesse: gesamt,
        ist_ordner: true,
        kategorie,
        kinder,
    })
}

/// Sucht einen Unterknoten anhand seines Pfads (für Breadcrumb-Zoom).
pub fn pfad_suchen<'a>(wurzel: &'a Knoten, ziel: &Path) -> Option<&'a Knoten> {
    if wurzel.pfad == ziel {
        return Some(wurzel);
    }
    for kind in &wurzel.kinder {
        if ziel.starts_with(&kind.pfad) {
            if let Some(treffer) = pfad_suchen(kind, ziel) {
                return Some(treffer);
            }
        }
    }
    None
}

/// Sammelt alle aufräumbaren Build-Artefakt-Ordner unterhalb eines Knotens.
pub fn build_artefakte_finden(wurzel: &Knoten) -> Vec<&Knoten> {
    let mut treffer = Vec::new();
    if wurzel.kategorie == Kategorie::BuildArtefakt {
        treffer.push(wurzel);
        return treffer;
    }
    for kind in &wurzel.kinder {
        treffer.extend(build_artefakte_finden(kind));
    }
    treffer.sort_by(|a, b| b.groesse.cmp(&a.groesse));
    treffer
}

/// Löscht ein Verzeichnis oder simuliert den Vorgang im Dry-Run-Modus.
pub fn ordner_bereinigen(pfad: &Path, knoten_groesse: u64, nur_test: bool) -> io::Result<u64> {
    if !pfad.exists() {
        return Ok(0);
    }
    if nur_test {
        return Ok(knoten_groesse);
    }
    fs::remove_dir_all(pfad)?;
    Ok(knoten_groesse)
}

/// Berechnet ein ausgewogenes Treemap-Layout per rekursiver Zweiteilung.
/// Kacheln unterhalb von `4.0` Pixeln werden verworfen (visuelle Klarheit).
pub fn treemap_berechnen(wurzel: &Knoten, flaeche: Bereich) -> Vec<Kachel> {
    let mut ergebnis = Vec::new();
    if wurzel.groesse == 0 || flaeche.breite < 4.0 || flaeche.hoehe < 4.0 {
        return ergebnis;
    }

    if wurzel.kinder.is_empty() {
        ergebnis.push(Kachel {
            name: wurzel.name.clone(),
            pfad: wurzel.pfad.clone(),
            groesse: wurzel.groesse,
            anteil: 100.0,
            ist_ordner: wurzel.ist_ordner,
            kategorie: wurzel.kategorie,
            bereich: flaeche,
        });
        return ergebnis;
    }

    let gesamt = wurzel.groesse as f32;
    treemap_teilen(&wurzel.kinder, flaeche, gesamt, &mut ergebnis);
    ergebnis
}

fn treemap_teilen(
    eintraege: &[Knoten],
    flaeche: Bereich,
    gesamt_wurzel: f32,
    ausgabe: &mut Vec<Kachel>,
) {
    if eintraege.is_empty() || flaeche.breite < 4.0 || flaeche.hoehe < 4.0 {
        return;
    }

    if eintraege.len() == 1 {
        let k = &eintraege[0];
        let anteil = if gesamt_wurzel > 0.0 {
            (k.groesse as f32 / gesamt_wurzel) * 100.0
        } else {
            0.0
        };
        ausgabe.push(Kachel {
            name: k.name.clone(),
            pfad: k.pfad.clone(),
            groesse: k.groesse,
            anteil,
            ist_ordner: k.ist_ordner,
            kategorie: k.kategorie,
            bereich: flaeche,
        });
        return;
    }

    let summe: u64 = eintraege.iter().map(|k| k.groesse).sum();
    if summe == 0 {
        return;
    }

    // Teile so nahe wie möglich an der Hälfte der Gesamtgröße, um quadratische Kacheln zu fördern.
    let ziel = summe / 2;
    let mut links_summe = 0u64;
    let mut schnitt = 1usize;

    for (idx, k) in eintraege.iter().enumerate() {
        links_summe += k.groesse;
        schnitt = idx + 1;
        if links_summe >= ziel && schnitt < eintraege.len() {
            break;
        }
    }

    let verhaeltnis = (links_summe as f32 / summe as f32).clamp(0.05, 0.95);
    let luecke = 2.0_f32.min(flaeche.breite * 0.1).min(flaeche.hoehe * 0.1);

    let (bereich_a, bereich_b) = if flaeche.breite >= flaeche.hoehe {
        let breite_a = (flaeche.breite - luecke) * verhaeltnis;
        let breite_b = (flaeche.breite - luecke) - breite_a;
        (
            Bereich {
                x: flaeche.x,
                y: flaeche.y,
                breite: breite_a,
                hoehe: flaeche.hoehe,
            },
            Bereich {
                x: flaeche.x + breite_a + luecke,
                y: flaeche.y,
                breite: breite_b,
                hoehe: flaeche.hoehe,
            },
        )
    } else {
        let hoehe_a = (flaeche.hoehe - luecke) * verhaeltnis;
        let hoehe_b = (flaeche.hoehe - luecke) - hoehe_a;
        (
            Bereich {
                x: flaeche.x,
                y: flaeche.y,
                breite: flaeche.breite,
                hoehe: hoehe_a,
            },
            Bereich {
                x: flaeche.x,
                y: flaeche.y + hoehe_a + luecke,
                breite: flaeche.breite,
                hoehe: hoehe_b,
            },
        )
    };

    treemap_teilen(&eintraege[..schnitt], bereich_a, gesamt_wurzel, ausgabe);
    treemap_teilen(&eintraege[schnitt..], bereich_b, gesamt_wurzel, ausgabe);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn beispiel_knoten(name: &str, groesse: u64, kinder: Vec<Knoten>) -> Knoten {
        let kategorie = if ist_build_artefakt_name(name) {
            Kategorie::BuildArtefakt
        } else if kinder.is_empty() {
            Kategorie::Quellcode
        } else {
            Kategorie::Ordner
        };
        Knoten {
            name: name.to_string(),
            pfad: PathBuf::from(format!("/projekt/{name}")),
            groesse,
            ist_ordner: !kinder.is_empty() || ist_build_artefakt_name(name),
            kategorie,
            kinder,
        }
    }

    #[test]
    fn groessen_formatierung_stimmt() {
        assert_eq!(format_groesse(512), "512 B");
        assert_eq!(format_groesse(2048), "2.0 KB");
        assert_eq!(format_groesse(5 * 1024 * 1024), "5.0 MB");
    }

    #[test]
    fn treemap_bleibt_innerhalb_der_zeichenflaeche() {
        let wurzel = beispiel_knoten(
            "root",
            1000,
            vec![
                beispiel_knoten("target", 600, vec![]),
                beispiel_knoten("src", 300, vec![]),
                beispiel_knoten("Cargo.toml", 100, vec![]),
            ],
        );

        let flaeche = Bereich {
            x: 0.0,
            y: 0.0,
            breite: 800.0,
            hoehe: 600.0,
        };
        let kacheln = treemap_berechnen(&wurzel, flaeche);
        assert_eq!(kacheln.len(), 3);

        for k in &kacheln {
            assert!(k.bereich.x >= 0.0);
            assert!(k.bereich.y >= 0.0);
            assert!(k.bereich.x + k.bereich.breite <= 800.1);
            assert!(k.bereich.y + k.bereich.hoehe <= 600.1);
        }
        assert_eq!(kacheln[0].kategorie, Kategorie::BuildArtefakt);
    }

    #[test]
    fn build_artefakte_werden_erkannt_und_sortiert() {
        let wurzel = beispiel_knoten(
            "workspace",
            900,
            vec![
                beispiel_knoten(
                    "app",
                    500,
                    vec![beispiel_knoten("target", 450, vec![])],
                ),
                beispiel_knoten("target", 400, vec![]),
            ],
        );

        let funde = build_artefakte_finden(&wurzel);
        assert_eq!(funde.len(), 2);
        assert_eq!(funde[0].groesse, 450);
        assert_eq!(funde[1].groesse, 400);
    }

    #[test]
    fn pfad_suchen_findet_unterordner() {
        let src = beispiel_knoten("src", 200, vec![]);
        let wurzel = beispiel_knoten("root", 200, vec![src.clone()]);
        let treffer = pfad_suchen(&wurzel, &src.pfad);
        assert_eq!(treffer.map(|k| k.name.as_str()), Some("src"));
    }

    #[test]
    fn scan_und_dry_run_auf_temp_ordner() {
        let basis = std::env::temp_dir().join("speicher_landkarte_test");
        let _ = fs::remove_dir_all(&basis);
        let target = basis.join("target");
        fs::create_dir_all(&target).unwrap();
        fs::write(target.join("cache.bin"), vec![1u8; 4096]).unwrap();
        fs::write(basis.join("main.rs"), "fn main() {}").unwrap();

        let baum = verzeichnis_scannen(&basis).unwrap();
        assert!(baum.groesse >= 4096);
        let artefakte = build_artefakte_finden(&baum);
        assert_eq!(artefakte.len(), 1);

        let gespart = ordner_bereinigen(&target, artefakte[0].groesse, true).unwrap();
        assert_eq!(gespart, 4096);
        assert!(target.exists(), "Dry-Run darf nicht löschen");

        let _ = fs::remove_dir_all(&basis);
    }
}
