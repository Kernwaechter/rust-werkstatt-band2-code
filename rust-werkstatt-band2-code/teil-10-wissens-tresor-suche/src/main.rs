//! Teil 11: Der Wissens-Tresor – Meisterprojekt
//! Die Rust-Werkstatt – Band 2: Native Desktop-Apps mit Iced
//! Autor: Volker Dähn

use iced::widget::{
    button, column, container, row, text, text_editor, text_input, Space,
};
use iced::{
    Alignment, Border, Color, Element, Length, Task,
};

mod db;
use db::{NotizEintrag, TresorDb};

fn main() -> iced::Result {
    iced::application(
        "Wissens-Tresor – Zettelkasten & Archiv",
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

struct TresorApp {
    db: Option<TresorDb>,
    notizen: Vec<NotizEintrag>,
    aktive_notiz: Option<NotizEintrag>,
    suchbegriff: String,
    editor_inhalt: text_editor::Content,
    status: String,
    aktiver_tag: Option<String>,
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
}

async fn db_starten() -> Result<TresorDb, String> {
    TresorDb::oeffnen("sqlite:tresor.db")
        .await
        .map_err(|e| e.to_string())
}

impl TresorApp {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
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
            Message::DbBereit(Err(err)) => {
                self.status = format!("DB-Fehler: {err}");
                Task::none()
            }
            Message::NotizenGeladen(Ok(liste)) => {
                let anzahl = liste.len();
                self.notizen = liste;
                self.status = format!("{anzahl} Notizen geladen");
                Task::none()
            }
            Message::NotizenGeladen(Err(err)) => {
                self.status = format!("Ladefehler: {err}");
                Task::none()
            }
            Message::SucheGeaendert(text) => {
                self.suchbegriff = text.clone();
                if let Some(datenbank) = self.db.clone() {
                    if text.trim().is_empty() {
                        Task::perform(
                            async move {
                                datenbank.alle_notizen().await
                                    .map_err(|e| e.to_string())
                            },
                            Message::NotizenGeladen,
                        )
                    } else {
                        Task::perform(
                            async move {
                                datenbank.fts_suchen(&text).await
                                    .map_err(|e| e.to_string())
                            },
                            Message::NotizenGeladen,
                        )
                    }
                } else {
                    Task::none()
                }
            }
            Message::TagAuswaehlen(tag) => {
                self.aktiver_tag = tag.clone();
                if let Some(t) = tag {
                    self.update(Message::SucheGeaendert(t))
                } else {
                    self.update(Message::SucheGeaendert(String::new()))
                }
            }
            Message::NotizAuswaehlen(notiz) => {
                self.editor_inhalt =
                    text_editor::Content::with_text(&notiz.inhalt);
                self.aktive_notiz = Some(notiz);
                Task::none()
            }
            Message::NeueNotiz => {
                let neuer_zettel = NotizEintrag {
                    id: 0,
                    titel: String::from("Neuer Zettel"),
                    inhalt: String::from("# Neuer Zettel\n\nGedanken hier festhalten...\n"),
                };
                let txt = &neuer_zettel.inhalt;
                self.editor_inhalt =
                    text_editor::Content::with_text(txt);
                self.aktive_notiz = Some(neuer_zettel);
                Task::none()
            }
            Message::NotizSpeichern => {
                if let (Some(datenbank), Some(notiz)) =
                    (self.db.clone(), &self.aktive_notiz)
                {
                    let inhalt = self.editor_inhalt.text();
                    let titel = inhalt
                        .lines()
                        .next()
                        .unwrap_or("Ohne Titel")
                        .trim_start_matches('#')
                        .trim()
                        .to_string();

                    let id = notiz.id;
                    Task::perform(
                        async move {
                            if id == 0 {
                                datenbank
                                    .notiz_einfuegen(&titel, &inhalt)
                                    .await
                                    .map(|_| ())
                                    .map_err(|e| e.to_string())
                            } else {
                                datenbank
                                    .notiz_aktualisieren(
                                        id, &titel, &inhalt,
                                    )
                                    .await
                                    .map_err(|e| e.to_string())
                            }
                        },
                        |_| Message::SucheGeaendert(String::new()),
                    )
                } else {
                    Task::none()
                }
            }
            Message::EditorAktion(aktion) => {
                self.editor_inhalt.perform(aktion);
                Task::none()
            }
        }
    }

    fn spalte_tags(&self) -> Element<'_, Message> {
        let ueberschrift = text("SCHLAGWORTE")
            .size(12)
            .color(Color::from_rgb(0.55, 0.60, 0.70));

        let tag_knopf = |tag: &'static str| {
            let ist_aktiv = self.aktiver_tag.as_deref() == Some(tag);
            button(text(tag).size(13))
                .on_press(Message::TagAuswaehlen(if ist_aktiv { None } else { Some(tag.to_string()) }))
                .padding([6, 12])
                .width(Length::Fill)
                .style(move |_, status| {
                    let mut stil = button::Style::default();
                    if ist_aktiv {
                        stil.background = Some(Color::from_rgba(0.96, 0.62, 0.07, 0.15).into());
                        stil.border = Border {
                            color: Color::from_rgb(0.96, 0.62, 0.07),
                            width: 1.0,
                            radius: 4.0.into(),
                        };
                    } else if status == button::Status::Hovered {
                        stil.background = Some(Color::from_rgba(1.0, 1.0, 1.0, 0.05).into());
                    }
                    stil
                })
        };

        let tag_liste = column![
            ueberschrift,
            tag_knopf("#rust"),
            tag_knopf("#iced"),
            tag_knopf("#security"),
            tag_knopf("#werkstatt"),
        ]
        .spacing(8);

        container(tag_liste)
            .width(160)
            .height(Length::Fill)
            .padding(16)
            .style(|_| container::Style {
                background: Some(Color::from_rgb(0.08, 0.10, 0.14).into()),
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
        let suchfeld = text_input("Notizen durchsuchen...", &self.suchbegriff)
            .on_input(Message::SucheGeaendert)
            .padding([8, 12]);

        let knopf_neu = button("+ Neuer Zettel")
            .on_press(Message::NeueNotiz)
            .width(Length::Fill)
            .padding([8, 16])
            .style(|_, status| {
                let mut stil = button::Style::default();
                if status == button::Status::Hovered {
                    stil.background = Some(Color::from_rgb(1.0, 0.70, 0.18).into());
                } else if status == button::Status::Pressed {
                    stil.background = Some(Color::from_rgb(0.80, 0.50, 0.05).into());
                } else {
                    stil.background = Some(Color::from_rgb(0.96, 0.62, 0.07).into());
                }
                stil.text_color = Color::from_rgb(0.08, 0.10, 0.14);
                stil
            });

        let mut liste = column![suchfeld, knopf_neu].spacing(12);

        for notiz in &self.notizen {
            let n_kopie = notiz.clone();
            let ist_aktiv = self.aktive_notiz.as_ref().map(|n| n.id) == Some(notiz.id);

            let vorschau = notiz.inhalt.lines().nth(1).unwrap_or("");
            let eintrag = button(
                column![
                    text(&notiz.titel)
                        .size(14)
                        .color(if ist_aktiv { Color::from_rgb(0.96, 0.62, 0.07) } else { Color::from_rgb(0.92, 0.94, 0.96) }),
                    text(vorschau)
                        .size(12)
                        .color(Color::from_rgb(0.55, 0.60, 0.70)),
                ]
                .spacing(4),
            )
            .on_press(Message::NotizAuswaehlen(n_kopie))
            .width(Length::Fill)
            .padding([8, 12])
            .style(move |_, status| {
                let mut stil = button::Style::default();
                if ist_aktiv {
                    stil.background = Some(Color::from_rgba(0.96, 0.62, 0.07, 0.12).into());
                    stil.border = Border {
                        color: Color::from_rgb(0.96, 0.62, 0.07),
                        width: 1.5,
                        radius: 6.0.into(),
                    };
                } else if status == button::Status::Hovered {
                    stil.background = Some(Color::from_rgba(1.0, 1.0, 1.0, 0.04).into());
                    stil.border = Border {
                        color: Color::from_rgb(0.24, 0.30, 0.40),
                        width: 1.0,
                        radius: 6.0.into(),
                    };
                } else {
                    stil.background = Some(Color::from_rgb(0.12, 0.15, 0.20).into());
                    stil.border = Border {
                        color: Color::from_rgb(0.18, 0.22, 0.28),
                        width: 1.0,
                        radius: 6.0.into(),
                    };
                }
                stil
            });

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
                .padding([6, 16])
                .style(|_, status| {
                    let mut stil = button::Style::default();
                    if status == button::Status::Hovered {
                        stil.background = Some(Color::from_rgb(1.0, 0.70, 0.18).into());
                    } else {
                        stil.background = Some(Color::from_rgb(0.96, 0.62, 0.07).into());
                    }
                    stil.text_color = Color::from_rgb(0.08, 0.10, 0.14);
                    stil
                }),
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

        let inhalt = column![
            kopf,
            editor_box,
            statusleiste,
        ]
        .spacing(12);

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
