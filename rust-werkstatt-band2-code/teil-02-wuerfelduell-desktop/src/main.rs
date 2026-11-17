//! Teil 2: Das Würfelduell Desktop
//! Die Rust-Werkstatt – Band 2: Native Desktop-Apps mit Iced
//! Autor: Volker Dähn

use iced::widget::{button, column, container, row, text, Space};
use iced::{Alignment, Border, Color, Element, Length};

fn main() -> iced::Result {
    iced::run(
        "Die Rust-Werkstatt – Würfelduell Desktop",
        WuerfelSpiel::update,
        WuerfelSpiel::view,
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SpielStatus {
    Bereit,
    PunktSpieler,
    PunktComputer,
    Unentschieden,
    SiegerSpieler,
    SiegerComputer,
}

struct WuerfelSpiel {
    spieler_wurf: u8,
    computer_wurf: u8,
    spieler_punkte: u32,
    computer_punkte: u32,
    runde: u32,
    status: SpielStatus,
}

impl Default for WuerfelSpiel {
    fn default() -> Self {
        Self {
            spieler_wurf: 1,
            computer_wurf: 1,
            spieler_punkte: 0,
            computer_punkte: 0,
            runde: 0,
            status: SpielStatus::Bereit,
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    Wuerfeln,
    NeuesSpiel,
}

const ZIEL_PUNKTE: u32 = 5;

impl WuerfelSpiel {
    fn update(&mut self, message: Message) {
        match message {
            Message::Wuerfeln => {
                if self.ist_beendet() {
                    return;
                }

                self.runde += 1;
                self.spieler_wurf = fastrand::u8(1..=6);
                self.computer_wurf = fastrand::u8(1..=6);

                if self.spieler_wurf > self.computer_wurf {
                    self.spieler_punkte += 1;
                    if self.spieler_punkte >= ZIEL_PUNKTE {
                        self.status = SpielStatus::SiegerSpieler;
                    } else {
                        self.status = SpielStatus::PunktSpieler;
                    }
                } else if self.computer_wurf > self.spieler_wurf {
                    self.computer_punkte += 1;
                    if self.computer_punkte >= ZIEL_PUNKTE {
                        self.status = SpielStatus::SiegerComputer;
                    } else {
                        self.status = SpielStatus::PunktComputer;
                    }
                } else {
                    self.status = SpielStatus::Unentschieden;
                }
            }
            Message::NeuesSpiel => {
                *self = Self::default();
            }
        }
    }

    fn ist_beendet(&self) -> bool {
        matches!(
            self.status,
            SpielStatus::SiegerSpieler | SpielStatus::SiegerComputer
        )
    }

    fn wuerfel_karte<'a>(
        name: &'a str,
        wurf: u8,
        punkte: u32,
        ist_fuehrend: bool,
    ) -> Element<'a, Message> {
        let name_text = text(name)
            .size(16)
            .style(|_theme| text::Style {
                color: Some(Color::from_rgb(0.58, 0.64, 0.72)),
            });

        let wurf_text = text(format!("{wurf}"))
            .size(52);

        let rahmen_farbe = if ist_fuehrend {
            Color::from_rgb(0.96, 0.62, 0.07) // Gold bei Führung/Sieg
        } else {
            Color::from_rgb(0.20, 0.28, 0.40) // Ruhiges Blaugrau
        };

        let karten_box = container(wurf_text)
            .width(110)
            .height(110)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(move |_theme| container::Style {
                background: Some(
                    Color::from_rgb(0.12, 0.16, 0.24).into(),
                ),
                border: Border {
                    color: rahmen_farbe,
                    width: if ist_fuehrend { 2.5 } else { 1.5 },
                    radius: 14.0.into(),
                },
                text_color: Some(Color::WHITE),
                ..container::Style::default()
            });

        let punkte_text = text(format!("{punkte} Punkte"))
            .size(14)
            .style(|_theme| text::Style {
                color: Some(Color::from_rgb(0.85, 0.88, 0.92)),
            });

        column![name_text, karten_box, punkte_text]
            .spacing(8)
            .align_x(Alignment::Center)
            .into()
    }

    fn view(&self) -> Element<'_, Message> {
        let titel = text("WÜRFELDUELL")
            .size(26)
            .style(|_theme| text::Style {
                color: Some(Color::WHITE),
            });

        let runden_anzeige = text(format!("RUNDE {}", self.runde))
            .size(12)
            .style(|_theme| text::Style {
                color: Some(Color::from_rgb(0.92, 0.35, 0.05)),
            });

        let spieler_fuehrt = self.spieler_punkte > self.computer_punkte
            || self.status == SpielStatus::SiegerSpieler;
        let computer_fuehrt = self.computer_punkte > self.spieler_punkte
            || self.status == SpielStatus::SiegerComputer;

        let spieler_karte = Self::wuerfel_karte(
            "Du (Spieler)",
            self.spieler_wurf,
            self.spieler_punkte,
            spieler_fuehrt,
        );

        let computer_karte = Self::wuerfel_karte(
            "Computer",
            self.computer_wurf,
            self.computer_punkte,
            computer_fuehrt,
        );

        let spielfeld = row![spieler_karte, computer_karte]
            .spacing(48)
            .align_y(Alignment::Center);

        let (status_text, status_farbe) = match self.status {
            SpielStatus::Bereit => {
                ("Erster zu 5 Punkten gewinnt!", Color::from_rgb(0.58, 0.64, 0.72))
            }
            SpielStatus::PunktSpieler => {
                ("Punkt für dich!", Color::from_rgb(0.22, 0.75, 0.45))
            }
            SpielStatus::PunktComputer => {
                ("Punkt für den Computer!", Color::from_rgb(0.94, 0.35, 0.35))
            }
            SpielStatus::Unentschieden => {
                ("Unentschieden!", Color::from_rgb(0.96, 0.70, 0.20))
            }
            SpielStatus::SiegerSpieler => {
                ("★ Glückwunsch! Du hast gewonnen! ★", Color::from_rgb(0.96, 0.62, 0.07))
            }
            SpielStatus::SiegerComputer => {
                ("Der Computer gewinnt die Partie!", Color::from_rgb(0.94, 0.35, 0.35))
            }
        };

        let status_meldung = text(status_text)
            .size(16)
            .style(move |_theme| text::Style {
                color: Some(status_farbe),
            });

        let mut inhalt = column![
            titel,
            runden_anzeige,
            Space::with_height(16),
            spielfeld,
            Space::with_height(16),
            status_meldung,
            Space::with_height(8),
        ]
        .spacing(12)
        .align_x(Alignment::Center);

        if !self.ist_beendet() {
            let knopf_wuerfeln = button(
                text("Jetzt würfeln")
                    .size(15)
                    .style(|_theme| text::Style {
                        color: Some(Color::WHITE),
                    }),
            )
            .on_press(Message::Wuerfeln)
            .padding([12, 32])
            .style(|_theme, status| {
                let basis = Color::from_rgb(0.92, 0.35, 0.05); // Werkstatt-Orange
                let hover = Color::from_rgb(1.0, 0.45, 0.15);
                let gedrueckt = Color::from_rgb(0.76, 0.26, 0.02);

                let hintergrund = match status {
                    button::Status::Hovered => hover,
                    button::Status::Pressed => gedrueckt,
                    _ => basis,
                };

                button::Style {
                    background: Some(hintergrund.into()),
                    border: Border {
                        radius: 10.0.into(),
                        ..Border::default()
                    },
                    ..button::Style::default()
                }
            });

            inhalt = inhalt.push(knopf_wuerfeln);
        }

        let knopf_neu = button(
            text(if self.ist_beendet() { "Neue Partie starten" } else { "Neues Spiel" })
                .size(13)
                .style(|_theme| text::Style {
                    color: Some(Color::from_rgb(0.70, 0.76, 0.84)),
                }),
        )
        .on_press(Message::NeuesSpiel)
        .padding([8, 20])
        .style(|_theme, status| {
            let rand_farbe = match status {
                button::Status::Hovered => Color::from_rgb(0.50, 0.60, 0.75),
                _ => Color::from_rgb(0.24, 0.32, 0.44),
            };

            button::Style {
                background: Some(Color::TRANSPARENT.into()),
                border: Border {
                    color: rand_farbe,
                    width: 1.0,
                    radius: 8.0.into(),
                },
                ..button::Style::default()
            }
        });

        inhalt = inhalt.push(knopf_neu);

        container(inhalt)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(|_theme| container::Style {
                background: Some(
                    Color::from_rgb(0.06, 0.09, 0.15).into(),
                ),
                ..container::Style::default()
            })
            .padding(24)
            .into()
    }
}
