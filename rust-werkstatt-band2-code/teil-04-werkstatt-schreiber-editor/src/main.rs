use iced::keyboard::{self, key::Named, Key, Modifiers};
use iced::widget::{
    button, column, container, row, text, text_editor, Space,
};
use iced::{event, Alignment, Border, Color, Element, Event, Length, Subscription};
use std::fs;
use std::path::PathBuf;

mod file_tree;
mod stats;
mod theme;

use file_tree::DateiEintrag;

fn main() -> iced::Result {
    iced::application("Werkstatt-Schreiber", App::update, App::view)
        .subscription(App::subscription)
        .run()
}

struct App {
    dateien: Vec<DateiEintrag>,
    aktive_datei: Option<PathBuf>,
    editor_inhalt: text_editor::Content,
    ist_geaendert: bool,
    status_meldung: String,
}

impl Default for App {
    fn default() -> Self {
        let aktuelle_dateien =
            file_tree::dateien_laden(&PathBuf::from("."));

        let demo_text = "# Willkommen im Werkstatt-Schreiber\n\n\
            Tippe deinen Text hier ein...";

        Self {
            dateien: aktuelle_dateien,
            aktive_datei: None,
            editor_inhalt: text_editor::Content::with_text(demo_text),
            ist_geaendert: false,
            status_meldung: String::from("Bereit"),
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    DateiAuswaehlen(PathBuf),
    EditorAktion(text_editor::Action),
    Speichern,
    Aktualisieren,
    TastaturEreignis(Event),
}

impl App {
    fn update(&mut self, message: Message) {
        match message {
            Message::DateiAuswaehlen(pfad) => {
                if let Ok(inhalt) = fs::read_to_string(&pfad) {
                    self.editor_inhalt =
                        text_editor::Content::with_text(&inhalt);
                    self.ist_geaendert = false;
                    let name = pfad
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy();
                    self.status_meldung = format!("Geladen: {name}");
                    self.aktive_datei = Some(pfad);
                } else {
                    self.status_meldung =
                        String::from("Fehler beim Lesen!");
                }
            }
            Message::EditorAktion(aktion) => {
                if aktion.is_edit() {
                    self.ist_geaendert = true;
                }
                self.editor_inhalt.perform(aktion);
            }
            Message::Speichern => {
                if let Some(ref pfad) = self.aktive_datei {
                    let text = self.editor_inhalt.text();
                    if fs::write(pfad, text).is_ok() {
                        self.ist_geaendert = false;
                        self.status_meldung =
                            String::from("Erfolgreich gespeichert!");
                    } else {
                        self.status_meldung =
                            String::from("Fehler beim Speichern!");
                    }
                } else {
                    self.status_meldung =
                        String::from("Keine aktive Datei!");
                }
            }
            Message::Aktualisieren => {
                let basis = PathBuf::from(".");
                self.dateien = file_tree::dateien_laden(&basis);
                self.status_meldung =
                    String::from("Dateiliste aktualisiert");
            }
            Message::TastaturEreignis(Event::Keyboard(
                keyboard::Event::KeyPressed {
                    key,
                    modifiers,
                    ..
                },
            )) if modifiers.contains(Modifiers::COMMAND)
                || modifiers.contains(Modifiers::CTRL) =>
            {
                match key {
                    Key::Named(Named::Save) => {
                        self.update(Message::Speichern);
                    }
                    Key::Character(ref c) if c == "s" || c == "S" => {
                        self.update(Message::Speichern);
                    }
                    _ => {}
                }
            }
            Message::TastaturEreignis(_) => {}
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        event::listen().map(Message::TastaturEreignis)
    }

    fn seitenleiste(&self) -> Element<'_, Message> {
        let ueberschrift = row![
            text("DATEIEN").size(14).color(theme::TEXT_GEDIKTET),
            Space::with_width(Length::Fill),
            button("Neu")
                .on_press(Message::Aktualisieren)
                .padding([6, 12])
        ]
        .align_y(Alignment::Center);

        let mut datei_liste = column![ueberschrift].spacing(8);

        for datei in &self.dateien {
            let ist_aktiv =
                self.aktive_datei.as_ref() == Some(&datei.pfad);
            let pfad_kopie = datei.pfad.clone();

            let knopf_text = text(&datei.name).size(13);
            let knopf = button(knopf_text)
                .on_press(Message::DateiAuswaehlen(pfad_kopie))
                .width(Length::Fill)
                .padding([8, 12])
                .style(move |_, status| {
                    let mut stil = button::Style::default();
                    if ist_aktiv {
                        stil.background =
                            Some(Color::from_rgba(0.96, 0.62, 0.07, 0.18).into());
                        stil.border = Border {
                            color: theme::AKZENT_GOLD,
                            width: 1.5,
                            radius: 4.0.into(),
                        };
                    } else if status == button::Status::Hovered {
                        stil.background =
                            Some(Color::from_rgba(1.0, 1.0, 1.0, 0.05).into());
                    }
                    stil
                });

            datei_liste = datei_liste.push(knopf);
        }

        container(datei_liste)
            .width(220)
            .height(Length::Fill)
            .padding(16)
            .style(|_| theme::seitenleisten_stil(&()))
            .into()
    }

    fn view(&self) -> Element<'_, Message> {
        let seitenleiste = self.seitenleiste();

        let datei_titel = match &self.aktive_datei {
            Some(p) => {
                let stern = if self.ist_geaendert { " *" } else { "" };
                format!("{}{stern}", p.display())
            }
            None => "Keine Datei geöffnet".to_string(),
        };

        let voller_text = self.editor_inhalt.text();
        let woerter = stats::woerter_zaehlen(&voller_text);
        let zeichen = stats::zeichen_zaehlen(&voller_text);
        let lesezeit = stats::lesezeit_minuten(woerter);

        let statusleiste = row![
            text(format!("{woerter} Wörter | {zeichen} Zeichen"))
                .size(12)
                .color(theme::TEXT_GEDIKTET),
            Space::with_width(Length::Fill),
            text(format!("Lesezeit: ~{lesezeit} Min."))
                .size(12)
                .color(theme::TEXT_GEDIKTET),
            Space::with_width(Length::Fixed(16.0)),
            text(&self.status_meldung)
                .size(12)
                .color(theme::AKZENT_GOLD),
            Space::with_width(Length::Fixed(16.0)),
            button("Speichern (Strg+S)")
                .on_press(Message::Speichern)
                .padding([4, 10]),
        ]
        .align_y(Alignment::Center)
        .padding([8, 16]);

        let editor_widget = text_editor(&self.editor_inhalt)
            .on_action(Message::EditorAktion)
            .padding(16);

        let editor_bereich = container(editor_widget)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_| container::Style {
                background: Some(theme::EDITOR_HINTERGRUND.into()),
                ..container::Style::default()
            });

        let haupt_spalte = column![
            row![
                text(datei_titel).size(16).color(theme::TEXT_HAUPT),
                Space::with_width(Length::Fill),
            ]
            .padding([16, 24]),
            editor_bereich,
            statusleiste,
        ];

        let haupt_container = container(haupt_spalte)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_| container::Style {
                background: Some(theme::HINTERGRUND.into()),
                ..container::Style::default()
            });

        row![seitenleiste, haupt_container].into()
    }
}
