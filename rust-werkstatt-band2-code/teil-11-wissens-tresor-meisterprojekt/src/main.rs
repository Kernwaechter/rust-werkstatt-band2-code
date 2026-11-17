//! Teil 11: Der Wissens-Tresor & Werkstatt-Auditor – Meisterprojekt
//! Die Rust-Werkstatt – Band 2: Native Desktop-Apps mit Iced
//! Autor: Volker Dähn

use std::path::PathBuf;

use iced::widget::{
    button, column, container, row, text, text_editor, text_input, Space,
};
use iced::{
    Alignment, Border, Color, Element, Length, Task,
};

mod db;
use db::{NotizEintrag, TresorDb};

mod auditor;
use auditor::AuditBefund;

fn main() -> iced::Result {
    iced::application(
        "Werkstatt-Auditor & Wissens-Tresor",
        TresorApp::update,
        TresorApp::view,
    )
    .run_with(|| {
        (
            TresorApp::default(),
            Task::perform(db_starten(), Message::DbBereit),
        )
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AnsichtModus {
    Zettelkasten,
    ProjektAuditor,
}

struct TresorApp {
    db: Option<TresorDb>,
    notizen: Vec<NotizEintrag>,
    aktive_notiz: Option<NotizEintrag>,
    suchbegriff: String,
    editor_inhalt: text_editor::Content,
    status: String,
    aktiver_tag: Option<String>,
    modus: AnsichtModus,
    // Auditor-Felder
    audit_pfad: String,
    befunde: Vec<AuditBefund>,
    aktiver_befund: Option<AuditBefund>,
    audit_laeuft: bool,
}

impl Default for TresorApp {
    fn default() -> Self {
        Self {
            db: None,
            notizen: Vec::new(),
            aktive_notiz: None,
            suchbegriff: String::new(),
            editor_inhalt: text_editor::Content::new(),
            status: String::from("Initialisiere Datenbank..."),
            aktiver_tag: None,
            modus: AnsichtModus::Zettelkasten,
            audit_pfad: String::from("."),
            befunde: Vec::new(),
            aktiver_befund: None,
            audit_laeuft: false,
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    DbBereit(Result<TresorDb, String>),
    NotizenGeladen(Result<Vec<NotizEintrag>, String>),
    SucheGeaendert(String),
    TagAuswaehlen(Option<String>),
    NotizAuswaehlen(NotizEintrag),
    NeueNotiz,
    NotizSpeichern,
    EditorAktion(text_editor::Action),
    // Auditor Messages
    ModusWechseln(AnsichtModus),
    AuditPfadGeaendert(String),
    AuditStarten,
    AuditFertig(Result<Vec<AuditBefund>, String>),
    BefundAuswaehlen(AuditBefund),
}

async fn db_starten() -> Result<TresorDb, String> {
    TresorDb::oeffnen("sqlite:tresor.db")
        .await
        .map_err(|e| e.to_string())
}

impl TresorApp {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ModusWechseln(neuer_modus) => {
                self.modus = neuer_modus;
                Task::none()
            }
            Message::DbBereit(Ok(datenbank)) => {
                self.db = Some(datenbank.clone());
                self.status = String::from("Bereit");
                Task::perform(
                    async move {
                        datenbank
                            .alle_notizen()
                            .await
                            .map_err(|e| e.to_string())
                    },
                    Message::NotizenGeladen,
                )
            }
            Message::DbBereit(Err(fehler)) => {
                self.status = format!("DB-Fehler: {fehler}");
                Task::none()
            }
            Message::NotizenGeladen(Ok(liste)) => {
                self.notizen = liste;
                self.status = format!("{} Einträge geladen", self.notizen.len());
                Task::none()
            }
            Message::NotizenGeladen(Err(fehler)) => {
                self.status = format!("Fehler beim Laden: {fehler}");
                Task::none()
            }
            Message::SucheGeaendert(begriff) => {
                self.suchbegriff = begriff.clone();
                let Some(datenbank) = self.db.clone() else {
                    return Task::none();
                };
                if begriff.trim().is_empty() {
                    Task::perform(
                        async move {
                            datenbank
                                .alle_notizen()
                                .await
                                .map_err(|e| e.to_string())
                        },
                        Message::NotizenGeladen,
                    )
                } else {
                    Task::perform(
                        async move {
                            datenbank
                                .fts_suchen(&begriff)
                                .await
                                .map_err(|e| e.to_string())
                        },
                        Message::NotizenGeladen,
                    )
                }
            }
            Message::TagAuswaehlen(tag) => {
                self.aktiver_tag = tag.clone();
                let Some(datenbank) = self.db.clone() else {
                    return Task::none();
                };
                match tag {
                    Some(t) => Task::perform(
                        async move {
                            datenbank
                                .fts_suchen(&t)
                                .await
                                .map_err(|e| e.to_string())
                        },
                        Message::NotizenGeladen,
                    ),
                    None => Task::perform(
                        async move {
                            datenbank
                                .alle_notizen()
                                .await
                                .map_err(|e| e.to_string())
                        },
                        Message::NotizenGeladen,
                    ),
                }
            }
            Message::NotizAuswaehlen(notiz) => {
                self.aktive_notiz = Some(notiz.clone());
                self.editor_inhalt = text_editor::Content::with_text(&notiz.inhalt);
                self.status = format!("Notiz „{}“ geladen", notiz.titel);
                Task::none()
            }
            Message::NeueNotiz => {
                self.aktive_notiz = None;
                self.editor_inhalt = text_editor::Content::with_text(
                    "# Neuer Zettel\n\nSchreibe hier deine Notiz...",
                );
                self.status = String::from("Neuer Entwurf erstellt");
                Task::none()
            }
            Message::NotizSpeichern => {
                let Some(datenbank) = self.db.clone() else {
                    self.status = String::from("Fehler: Keine Datenbankverbindung");
                    return Task::none();
                };
                let text = self.editor_inhalt.text();
                let titel = text
                    .lines()
                    .next()
                    .unwrap_or("Ohne Titel")
                    .trim_start_matches('#')
                    .trim()
                    .to_string();

                let aktive_id = self.aktive_notiz.as_ref().map(|n| n.id);

                Task::perform(
                    async move {
                        if let Some(id) = aktive_id {
                            datenbank.notiz_aktualisieren(id, &titel, &text).await
                        } else {
                            datenbank.notiz_einfuegen(&titel, &text).await.map(|_| ())
                        }
                        .map_err(|e| e.to_string())
                    },
                    |ergebnis| match ergebnis {
                        Ok(()) => Message::SucheGeaendert(String::new()),
                        Err(fehler) => Message::NotizenGeladen(Err(fehler)),
                    },
                )
            }
            Message::EditorAktion(aktion) => {
                self.editor_inhalt.perform(aktion);
                Task::none()
            }
            // Auditor-Events
            Message::AuditPfadGeaendert(p) => {
                self.audit_pfad = p;
                Task::none()
            }
            Message::AuditStarten => {
                self.audit_laeuft = true;
                self.status = String::from("Sicherheitsprüfung läuft...");
                let p = PathBuf::from(&self.audit_pfad);
                Task::perform(
                    async move { auditor::scanne_projekt(&p) },
                    Message::AuditFertig,
                )
            }
            Message::AuditFertig(Ok(befunde)) => {
                self.audit_laeuft = false;
                self.status = format!("Audit abgeschlossen: {} Befunde gefunden.", befunde.len());
                self.befunde = befunde;
                self.aktiver_befund = self.befunde.first().cloned();
                Task::none()
            }
            Message::AuditFertig(Err(fehler)) => {
                self.audit_laeuft = false;
                self.status = format!("Audit-Fehler: {fehler}");
                Task::none()
            }
            Message::BefundAuswaehlen(befund) => {
                self.aktiver_befund = Some(befund);
                Task::none()
            }
        }
    }

    fn spalte_tags(&self) -> Element<'_, Message> {
        let titel = text("WERKSTATT")
            .size(12)
            .color(Color::from_rgb(0.55, 0.60, 0.70));

        let btn_zettel = button("📝 Zettelkasten")
            .width(Length::Fill)
            .padding([8, 12])
            .on_press(Message::ModusWechseln(AnsichtModus::Zettelkasten));

        let btn_audit = button("🛡️ Projekt-Auditor")
            .width(Length::Fill)
            .padding([8, 12])
            .on_press(Message::ModusWechseln(AnsichtModus::ProjektAuditor));

        let mut inhalt = column![titel, btn_zettel, btn_audit]
            .spacing(8)
            .width(Length::Fill);

        if self.modus == AnsichtModus::Zettelkasten {
            let tags_titel = text("SCHLAGWÖRTER")
                .size(11)
                .color(Color::from_rgb(0.50, 0.55, 0.65));
            inhalt = inhalt.push(Space::with_height(16)).push(tags_titel);

            let alle_btn = button("Alle Notizen")
                .width(Length::Fill)
                .padding([6, 12])
                .on_press(Message::TagAuswaehlen(None));
            inhalt = inhalt.push(alle_btn);

            let tags = ["#rust", "#iced", "#sqlite", "#async", "#clean-code"];
            for tag in tags {
                let btn = button(text(tag).size(13))
                    .width(Length::Fill)
                    .padding([6, 12])
                    .on_press(Message::TagAuswaehlen(Some(tag.to_string())));
                inhalt = inhalt.push(btn);
            }
        }

        container(inhalt)
            .width(180)
            .height(Length::Fill)
            .padding(16)
            .style(|_| container::Style {
                background: Some(Color::from_rgb(0.08, 0.10, 0.13).into()),
                border: Border {
                    color: Color::from_rgb(0.18, 0.22, 0.28),
                    width: 1.0,
                    radius: 0.0.into(),
                },
                ..container::Style::default()
            })
            .into()
    }

    fn spalte_notizen(&self) -> Element<'_, Message> {
        if self.modus == AnsichtModus::ProjektAuditor {
            let kopf = column![
                text("Projekt-Auditor").size(15).color(Color::WHITE),
                text_input("Pfad zum Projekt (z.B. .)", &self.audit_pfad)
                    .on_input(Message::AuditPfadGeaendert)
                    .padding(8),
                button(if self.audit_laeuft { "Prüfe..." } else { "Scan starten" })
                    .on_press(Message::AuditStarten)
                    .padding([6, 12]),
            ]
            .spacing(8);

            let mut liste = column![kopf].spacing(8);

            for b in &self.befunde {
                let badge = container(text(b.schweregrad.text()).size(10).color(Color::WHITE))
                    .padding([2, 6])
                    .style({
                        let farbe = b.schweregrad.badge_farbe();
                        move |_| container::Style {
                            background: Some(farbe.into()),
                            border: Border { radius: 4.0.into(), ..Border::default() },
                            ..container::Style::default()
                        }
                    });

                let karte = button(
                    column![
                        row![badge, text(&b.kategorie).size(11).color(Color::from_rgb(0.6, 0.7, 0.8))].spacing(8),
                        text(&b.titel).size(13).color(Color::WHITE),
                    ]
                    .spacing(4),
                )
                .width(Length::Fill)
                .padding(8)
                .on_press(Message::BefundAuswaehlen(b.clone()));

                liste = liste.push(karte);
            }

            return container(liste)
                .width(300)
                .height(Length::Fill)
                .padding(16)
                .style(|_| container::Style {
                    background: Some(Color::from_rgb(0.10, 0.12, 0.16).into()),
                    ..container::Style::default()
                })
                .into();
        }

        // Zettelkasten-Liste
        let suchfeld = text_input("Notizen durchsuchen...", &self.suchbegriff)
            .on_input(Message::SucheGeaendert)
            .padding(10);

        let neuer_knopf = button("+ Neuer Zettel")
            .width(Length::Fill)
            .padding([8, 16])
            .on_press(Message::NeueNotiz);

        let mut liste = column![suchfeld, neuer_knopf].spacing(10);

        for notiz in &self.notizen {
            let eintrag = button(
                column![
                    text(&notiz.titel).size(14).color(Color::from_rgb(0.95, 0.96, 0.98)),
                    text(notiz.inhalt.lines().nth(1).unwrap_or("")).size(11).color(Color::from_rgb(0.55, 0.60, 0.70)),
                ]
                .spacing(4),
            )
            .width(Length::Fill)
            .padding([8, 12])
            .on_press(Message::NotizAuswaehlen(notiz.clone()));

            liste = liste.push(eintrag);
        }

        container(liste)
            .width(280)
            .height(Length::Fill)
            .padding(16)
            .style(|_| container::Style {
                background: Some(Color::from_rgb(0.10, 0.12, 0.16).into()),
                border: Border {
                    color: Color::from_rgb(0.18, 0.22, 0.28),
                    width: 1.0,
                    radius: 0.0.into(),
                },
                ..container::Style::default()
            })
            .into()
    }

    fn spalte_editor(&self) -> Element<'_, Message> {
        if self.modus == AnsichtModus::ProjektAuditor {
            let Some(b) = &self.aktiver_befund else {
                return container(text("Wähle links einen Befund für Details.").color(Color::from_rgb(0.5, 0.6, 0.7)))
                    .center_x(Length::Fill)
                    .center_y(Length::Fill)
                    .into();
            };

            let mut info_zeile = row![
                text(format!("Schweregrad: {}", b.schweregrad.text())).color(b.schweregrad.badge_farbe()),
                Space::with_width(16),
                text(format!("Kategorie: {}", b.kategorie)).color(Color::from_rgb(0.7, 0.8, 0.9)),
            ];

            if let Some(cve) = &b.cve_id {
                info_zeile = info_zeile.push(Space::with_width(16)).push(
                    text(format!("Advisory: {cve}")).color(Color::from_rgb(1.0, 0.4, 0.4)),
                );
            }

            let mut fund_details = format!("Datei: {}", b.dateipfad.display());
            if let Some(zn) = b.zeilennummer {
                fund_details.push_str(&format!(" (Zeile {zn})"));
            }

            let inhalt = column![
                text(format!("🛡️ Befund: {}", b.titel)).size(20).color(Color::WHITE),
                info_zeile,
                Space::with_height(16),
                text("Beschreibung & Risiko:").size(14).color(Color::from_rgb(0.8, 0.85, 0.9)),
                text(&b.details).size(13).color(Color::from_rgb(0.7, 0.75, 0.8)),
                Space::with_height(16),
                text("Fundstelle:").size(14).color(Color::from_rgb(0.8, 0.85, 0.9)),
                text(fund_details).size(12).color(Color::from_rgb(0.9, 0.7, 0.2)),
            ]
            .spacing(8);

            return container(inhalt)
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(24)
                .style(|_| container::Style {
                    background: Some(Color::from_rgb(0.06, 0.08, 0.11).into()),
                    ..container::Style::default()
                })
                .into();
        }

        if self.aktive_notiz.is_none() {
            let hinweis = "← Wähle links eine Notiz aus oder erstelle einen neuen Zettel.";
            let farbe = Color::from_rgb(0.55, 0.60, 0.70);
            return container(text(hinweis).color(farbe).size(14))
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .style(|_| container::Style {
                    background: Some(Color::from_rgb(0.06, 0.08, 0.11).into()),
                    ..container::Style::default()
                })
                .into();
        }

        let kopf = row![
            text("Zettel bearbeiten").size(16).color(Color::from_rgb(0.92, 0.94, 0.96)),
            Space::with_width(Length::Fill),
            button("Speichern (Strg+S)")
                .on_press(Message::NotizSpeichern)
                .padding([6, 16]),
        ]
        .align_y(Alignment::Center)
        .padding([8, 0]);

        let editor = text_editor(&self.editor_inhalt)
            .on_action(Message::EditorAktion)
            .padding(16);

        let editor_box = container(editor)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_| container::Style {
                background: Some(Color::from_rgb(0.08, 0.10, 0.14).into()),
                border: Border {
                    color: Color::from_rgb(0.18, 0.22, 0.28),
                    width: 1.0,
                    radius: 6.0.into(),
                },
                ..container::Style::default()
            });

        let statusleiste = row![
            text(&self.status).size(12).color(Color::from_rgb(0.55, 0.60, 0.70)),
            Space::with_width(Length::Fill),
            text(format!("{} Zeichen", self.editor_inhalt.text().chars().count()))
                .size(12)
                .color(Color::from_rgb(0.55, 0.60, 0.70)),
        ]
        .align_y(Alignment::Center)
        .padding([4, 0]);

        let inhalt = column![kopf, editor_box, statusleiste].spacing(12);

        container(inhalt)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(24)
            .style(|_| container::Style {
                background: Some(Color::from_rgb(0.06, 0.08, 0.11).into()),
                ..container::Style::default()
            })
            .into()
    }

    fn view(&self) -> Element<'_, Message> {
        row![
            self.spalte_tags(),
            self.spalte_notizen(),
            self.spalte_editor(),
        ]
        .into()
    }
}
