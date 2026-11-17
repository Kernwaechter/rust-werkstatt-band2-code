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
use theme::ThemaModus;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RibbonTab {
    #[default]
    Datei,
    Format,
    Ansicht,
}

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
    thema: ThemaModus,
    aktiver_tab: RibbonTab,
    statusleiste_sichtbar: bool,
}

impl Default for App {
    fn default() -> Self {
        let dateien = file_tree::dateien_laden(&PathBuf::from("."));

        let demo_text = "# Gedanken an der Werkbank\n\n\
            Es gibt Tage, an denen ein Werkstück sofort gelingt,\n\
            und Tage, an denen der Hobel hakt.\n\n\
            Beim Programmieren ist das nicht anders: Wer mit Ruhe,\n\
            sauberen Modulen und Sorgfalt an den Quellcode herangeht,\n\
            baut Software, die Jahrzehnte übersteht.";

        Self {
            dateien,
            aktive_datei: Some(PathBuf::from("notizen.md")),
            editor_inhalt: text_editor::Content::with_text(demo_text),
            ist_geaendert: false,
            status_meldung: String::from("Bereit"),
            thema: ThemaModus::Dunkel,
            aktiver_tab: RibbonTab::Datei,
            statusleiste_sichtbar: true,
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    DateiAuswaehlen(PathBuf),
    EditorAktion(text_editor::Action),
    Speichern,
    Aktualisieren,
    ThemaUmschalten,
    TastaturEreignis(Event),
    TabAuswaehlen(RibbonTab),
    EinfuegenText(&'static str),
    StatusleisteUmschalten,
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
                            String::from("Gespeichert");
                    } else {
                        self.status_meldung =
                            String::from("Fehler beim Speichern!");
                    }
                } else {
                    self.status_meldung =
                        String::from("Keine Datei gewählt!");
                }
            }
            Message::Aktualisieren => {
                let basis = PathBuf::from(".");
                self.dateien = file_tree::dateien_laden(&basis);
                self.status_meldung =
                    String::from("Dateiliste aktualisiert");
            }
            Message::ThemaUmschalten => {
                self.thema = self.thema.umschalten();
            }
            Message::TabAuswaehlen(tab) => {
                self.aktiver_tab = tab;
            }
            Message::EinfuegenText(schnipsel) => {
                self.editor_inhalt.perform(text_editor::Action::Edit(
                    text_editor::Edit::Paste(schnipsel.to_string().into()),
                ));
                self.ist_geaendert = true;
            }
            Message::StatusleisteUmschalten => {
                self.statusleiste_sichtbar = !self.statusleiste_sichtbar;
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
        let th = self.thema;

        let ueberschrift = row![
            text("DATEIEN").size(13).color(th.text_gedimmt()),
            Space::with_width(Length::Fill),
            button("Neu")
                .on_press(Message::Aktualisieren)
                .padding([3, 7])
        ]
        .align_y(Alignment::Center);

        let mut datei_liste = column![ueberschrift].spacing(6);

        for datei in &self.dateien {
            let ist_aktiv =
                self.aktive_datei.as_ref() == Some(&datei.pfad);
            let pfad_kopie = datei.pfad.clone();

            let knopf_text = text(&datei.name).size(13);
            let knopf = button(knopf_text)
                .on_press(Message::DateiAuswaehlen(pfad_kopie))
                .width(Length::Fill)
                .padding([6, 10])
                .style(move |_, status| {
                    let mut stil = button::Style::default();
                    if ist_aktiv {
                        stil.background =
                            Some(Color::from_rgba(0.96, 0.62, 0.07, 0.18).into());
                        stil.border = Border {
                            color: th.akzent_gold(),
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
            .padding(14)
            .style(move |_| theme::seitenleisten_stil(th))
            .into()
    }

    fn ribbon_bar(&self) -> Element<'_, Message> {
        let th = self.thema;

        let tab_knopf = |tab: RibbonTab, titel: &'static str| {
            let ist_aktiv = self.aktiver_tab == tab;
            button(text(titel).size(12))
                .on_press(Message::TabAuswaehlen(tab))
                .padding([4, 12])
                .style(move |_, status| {
                    let mut stil = button::Style::default();
                    if ist_aktiv {
                        stil.background = Some(Color::from_rgba(0.96, 0.62, 0.07, 0.15).into());
                        stil.border = Border {
                            color: th.akzent_gold(),
                            width: 1.5,
                            radius: 4.0.into(),
                        };
                    } else if status == button::Status::Hovered {
                        stil.background = Some(Color::from_rgba(1.0, 1.0, 1.0, 0.05).into());
                    } else {
                        stil.background = None;
                    }
                    stil
                })
        };

        let tabs = row![
            tab_knopf(RibbonTab::Datei, "Datei"),
            tab_knopf(RibbonTab::Format, "Formatierung"),
            tab_knopf(RibbonTab::Ansicht, "Ansicht"),
        ]
        .spacing(4);

        let tab_inhalt: Element<'_, Message> = match self.aktiver_tab {
            RibbonTab::Datei => row![
                self.ribbon_gruppe(
                    "Dokument",
                    row![
                        button(text("Speichern")).on_press(Message::Speichern).padding([4, 8]),
                        button(text("Neu laden")).on_press(Message::Aktualisieren).padding([4, 8]),
                    ]
                    .spacing(6),
                ),
                self.ribbon_gruppe(
                    "Auswahl",
                    row![
                        button(text("Alles auswählen"))
                            .on_press(Message::EditorAktion(text_editor::Action::SelectAll))
                            .padding([4, 8]),
                        button(text("Zeile wählen"))
                            .on_press(Message::EditorAktion(text_editor::Action::SelectLine))
                            .padding([4, 8]),
                    ]
                    .spacing(6),
                ),
            ]
            .spacing(12)
            .into(),

            RibbonTab::Format => row![
                self.ribbon_gruppe(
                    "Überschriften",
                    row![
                        button(text("H1 (#)")).on_press(Message::EinfuegenText("\n# ")).padding([4, 8]),
                        button(text("H2 (##)")).on_press(Message::EinfuegenText("\n## ")).padding([4, 8]),
                        button(text("H3 (###)")).on_press(Message::EinfuegenText("\n### ")).padding([4, 8]),
                    ]
                    .spacing(6),
                ),
                self.ribbon_gruppe(
                    "Auszeichnung",
                    row![
                        button(text("Fett (**)")).on_press(Message::EinfuegenText("****")).padding([4, 8]),
                        button(text("Kursiv (*)")).on_press(Message::EinfuegenText("**")).padding([4, 8]),
                        button(text("Code (`)")).on_press(Message::EinfuegenText("``")).padding([4, 8]),
                        button(text("Block (```)")).on_press(Message::EinfuegenText("\n```rust\n\n```\n")).padding([4, 8]),
                    ]
                    .spacing(6),
                ),
            ]
            .spacing(12)
            .into(),

            RibbonTab::Ansicht => row![
                self.ribbon_gruppe(
                    "Darstellung",
                    row![
                        button(text(match th {
                            ThemaModus::Dunkel => "☀ Helles Thema",
                            ThemaModus::Hell => "🌙 Dunkles Thema",
                        }))
                        .on_press(Message::ThemaUmschalten)
                        .padding([4, 8]),
                    ],
                ),
                self.ribbon_gruppe(
                    "Elemente",
                    row![
                        button(text(if self.statusleiste_sichtbar {
                            "Statuszeile: An"
                        } else {
                            "Statuszeile: Aus"
                        }))
                        .on_press(Message::StatusleisteUmschalten)
                        .padding([4, 8]),
                    ],
                ),
            ]
            .spacing(12)
            .into(),
        };

        container(
            column![
                tabs,
                container(tab_inhalt)
                    .width(Length::Fill)
                    .padding([6, 10])
                    .style(move |_| container::Style {
                        background: Some(th.editor_hintergrund().into()),
                        border: Border {
                            color: th.trennlinie(),
                            width: 1.0,
                            radius: 4.0.into(),
                        },
                        ..container::Style::default()
                    })
            ]
            .spacing(2),
        )
        .padding([4, 16])
        .into()
    }

    fn ribbon_gruppe<'a>(
        &self,
        titel: &'a str,
        inhalt: impl Into<Element<'a, Message>>,
    ) -> Element<'a, Message> {
        let th = self.thema;
        container(
            column![
                inhalt.into(),
                Space::with_height(Length::Fixed(4.0)),
                text(titel).size(10).color(th.text_gedimmt()),
            ]
            .align_x(Alignment::Center),
        )
        .padding([2, 6])
        .style(move |_| container::Style {
            border: Border {
                color: Color::from_rgba(1.0, 1.0, 1.0, 0.04),
                width: 1.0,
                radius: 4.0.into(),
            },
            ..container::Style::default()
        })
        .into()
    }

    fn view(&self) -> Element<'_, Message> {
        let th = self.thema;
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

        let thema_knopf_beschriftung = match th {
            ThemaModus::Dunkel => "☀ Hell",
            ThemaModus::Hell => "🌙 Dunkel",
        };

        let statusleiste = row![
            text(format!("{woerter} Wörter | {zeichen} Zeichen"))
                .size(12)
                .color(th.text_gedimmt()),
            Space::with_width(Length::Fixed(16.0)),
            text(format!("Lesezeit: ~{lesezeit} Min."))
                .size(12)
                .color(th.text_gedimmt()),
            Space::with_width(Length::Fill),
            text(&self.status_meldung)
                .size(12)
                .color(th.akzent_gold()),
            Space::with_width(Length::Fixed(16.0)),
            button(thema_knopf_beschriftung)
                .on_press(Message::ThemaUmschalten)
                .padding([4, 10]),
            Space::with_width(Length::Fixed(8.0)),
            button("Speichern (Strg+S)")
                .on_press(Message::Speichern)
                .padding([4, 10]),
        ]
        .align_y(Alignment::Center)
        .padding([8, 16]);

        let editor_widget = text_editor(&self.editor_inhalt)
            .on_action(Message::EditorAktion)
            .padding(20);

        let zentrierter_editor = container(editor_widget)
            .max_width(750)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |_| container::Style {
                background: Some(th.editor_hintergrund().into()),
                border: Border {
                    color: th.trennlinie(),
                    width: 1.0,
                    radius: 6.0.into(),
                },
                ..container::Style::default()
            });

        let mut haupt_spalte = column![
            row![
                text(datei_titel).size(16).color(th.text_haupt()),
                Space::with_width(Length::Fill),
            ]
            .padding([8, 16]),
            self.ribbon_bar(),
            container(zentrierter_editor)
                .width(Length::Fill)
                .height(Length::Fill)
                .padding([8, 16]),
        ];

        if self.statusleiste_sichtbar {
            haupt_spalte = haupt_spalte.push(statusleiste);
        }

        let haupt_container = container(haupt_spalte)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |_| container::Style {
                background: Some(th.hintergrund().into()),
                ..container::Style::default()
            });

        row![seitenleiste, haupt_container].into()
    }
}
