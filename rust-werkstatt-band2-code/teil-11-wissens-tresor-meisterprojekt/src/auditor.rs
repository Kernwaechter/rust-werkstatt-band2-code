//! Der Werkstatt-Auditor: Sicherheits- und Supply-Chain-Prüfung für Rust-Projekte
//! Die Rust-Werkstatt – Band 2: Native Desktop-Apps mit Iced
//! Autor: Volker Dähn

use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum Schweregrad {
    Kritisch,
    Warnung,
    Hinweis,
}

impl Schweregrad {
    pub fn badge_farbe(&self) -> iced::Color {
        match self {
            Self::Kritisch => iced::Color::from_rgb(0.9, 0.2, 0.2), // Rot
            Self::Warnung => iced::Color::from_rgb(0.95, 0.65, 0.1), // Bernstein
            Self::Hinweis => iced::Color::from_rgb(0.2, 0.6, 0.9),  // Blau
        }
    }

    pub fn text(&self) -> &'static str {
        match self {
            Self::Kritisch => "KRITISCH",
            Self::Warnung => "WARNUNG",
            Self::Hinweis => "HINWEIS",
        }
    }
}

#[derive(Debug, Clone)]
pub struct AuditBefund {
    pub schweregrad: Schweregrad,
    pub kategorie: String,
    pub cve_id: Option<String>,
    pub titel: String,
    pub details: String,
    pub dateipfad: PathBuf,
    pub zeilennummer: Option<usize>,
}

/// Führt ein umfassendes Sicherheits-Audit für ein lokales Rust-Projekt durch.
pub fn scanne_projekt(projekt_pfad: &Path) -> Result<Vec<AuditBefund>, String> {
    if !projekt_pfad.exists() {
        return Err(format!("Projektpfad „{}“ existiert nicht.", projekt_pfad.display()));
    }

    let mut befunde = Vec::new();

    // 1. Cargo.lock auf bekannte RustSec-Schwachstellen prüfen
    let lock_pfad = projekt_pfad.join("Cargo.lock");
    if lock_pfad.exists() {
        if let Ok(lock_inhalt) = fs::read_to_string(&lock_pfad) {
            pruefe_cargo_lock(&lock_inhalt, &lock_pfad, &mut befunde);
        }
    }

    // 2. Quelltext in src/ rekursiv auf unsafe und Secrets prüfen
    let src_pfad = projekt_pfad.join("src");
    if src_pfad.exists() {
        scan_quellcode_rekursiv(&src_pfad, &mut befunde);
    }

    // 3. Auf herumliegende .env oder Schlüsseldateien prüfen
    pruefe_sensible_dateien(projekt_pfad, &mut befunde);

    Ok(befunde)
}

fn pruefe_cargo_lock(lock_inhalt: &str, lock_pfad: &Path, befunde: &mut Vec<AuditBefund>) {
    // Bekannte RustSec Advisory Signaturen (Muster für Crate-Versionen)
    let datenbank: &[(&str, &str, &str, &str, Schweregrad)] = &[
        (
            "time",
            "version = \"0.1.",
            "RUSTSEC-2020-0071",
            "Mögliche Speicherbeschädigung durch unsicheren Zeitzonen-Zugriff in time 0.1.x. Update auf >= 0.3 erforderlich.",
            Schweregrad::Kritisch,
        ),
        (
            "atty",
            "version = \"0.2.",
            "RUSTSEC-2021-0145",
            "Unaligned memory read in atty unter Windows. Ersatz durch std::io::IsTerminal empfohlen.",
            Schweregrad::Warnung,
        ),
        (
            "chrono",
            "version = \"0.4.19\"",
            "RUSTSEC-2020-0159",
            "Segfault in localtime_r auf Unix-Systemen. Update auf chrono >= 0.4.20 erforderlich.",
            Schweregrad::Kritisch,
        ),
    ];

    for (crate_name, versions_muster, id, beschreibung, schwere) in datenbank {
        if lock_inhalt.contains(&format!("name = \"{crate_name}\"")) && lock_inhalt.contains(versions_muster) {
            befunde.push(AuditBefund {
                schweregrad: schwere.clone(),
                kategorie: "RustSec Advisory".into(),
                cve_id: Some(id.to_string()),
                titel: format!("Verwundbare Crate entdeckt: {crate_name}"),
                details: beschreibung.to_string(),
                dateipfad: lock_pfad.to_path_buf(),
                zeilennummer: None,
            });
        }
    }
}

fn scan_quellcode_rekursiv(pfad: &Path, befunde: &mut Vec<AuditBefund>) {
    let eintraege = match fs::read_dir(pfad) {
        Ok(e) => e,
        Err(_) => return,
    };

    for eintrag in eintraege.flatten() {
        let p = eintrag.path();
        if p.is_dir() {
            scan_quellcode_rekursiv(&p, befunde);
        } else if p.extension().is_some_and(|ext| ext == "rs") {
            pruefe_quellcodedatei(&p, befunde);
        }
    }
}

fn pruefe_quellcodedatei(datei: &Path, befunde: &mut Vec<AuditBefund>) {
    let inhalt = match fs::read_to_string(datei) {
        Ok(i) => i,
        Err(_) => return,
    };

    for (zeile_idx, zeile) in inhalt.lines().enumerate() {
        let zeile_nr = zeile_idx + 1;
        let getrimmt = zeile.trim();

        // 1. Unsafe-Radar
        if getrimmt.starts_with("unsafe ") || getrimmt.contains(" unsafe ") || getrimmt == "unsafe {" {
            befunde.push(AuditBefund {
                schweregrad: Schweregrad::Warnung,
                kategorie: "Unsafe-Radar".into(),
                cve_id: None,
                titel: "Unsafe-Block im Quelltext".into(),
                details: "Unsafe-Code umgeht Rusts Speichergarantien und verlangt manuelle Verifikation.".into(),
                dateipfad: datei.to_path_buf(),
                zeilennummer: Some(zeile_nr),
            });
        }

        // 2. Secret-Finder
        if getrimmt.contains("AKIA") && getrimmt.len() >= 20 {
            befunde.push(AuditBefund {
                schweregrad: Schweregrad::Kritisch,
                kategorie: "Secret-Leak".into(),
                cve_id: None,
                titel: "Verdächtiger AWS-Access-Key gefunden".into(),
                details: "Ein möglicher AWS API-Schlüssel wurde hart im Quelltext codiert.".into(),
                dateipfad: datei.to_path_buf(),
                zeilennummer: Some(zeile_nr),
            });
        } else if getrimmt.contains("ghp_") {
            befunde.push(AuditBefund {
                schweregrad: Schweregrad::Kritisch,
                kategorie: "Secret-Leak".into(),
                cve_id: None,
                titel: "GitHub Personal Access Token gefunden".into(),
                details: "Privater GitHub-Token im Code entdeckt. Sofort widerrufen und in .env auslagern!".into(),
                dateipfad: datei.to_path_buf(),
                zeilennummer: Some(zeile_nr),
            });
        }
    }
}

fn pruefe_sensible_dateien(wurzel: &Path, befunde: &mut Vec<AuditBefund>) {
    let verdaechtig = &[".env", "id_rsa", "credentials.json", "private.key"];
    for dateiname in verdaechtig {
        let p = wurzel.join(dateiname);
        if p.exists() {
            befunde.push(AuditBefund {
                schweregrad: Schweregrad::Warnung,
                kategorie: "Dateisicherheit".into(),
                cve_id: None,
                titel: format!("Sensible Datei im Projektordner: {dateiname}"),
                details: "Diese Datei sollte zwingend in .gitignore eingetragen und nicht committed werden.".into(),
                dateipfad: p,
                zeilennummer: None,
            });
        }
    }
}
