use iced::widget::{button, column, container, row, text, Space};
use iced::{Alignment, Border, Color, Element, Length};
use std::path::PathBuf;

mod file_tree;
mod stats;
mod theme;

use file_tree::DateiEintrag;

fn main() -> iced::Result {
    iced::run("Werkstatt-Schreiber", App::update, App::view)
}

struct App {
    dateien: Vec<DateiEintrag>,
    aktive_datei: Option<PathBuf>,
    status_meldung: String,
}

impl Default for App {
    fn default() -> Self {
        let aktuelle_dateien =
            file_tree::dateien_laden(&PathBuf::from("."));

        Self {
            dateien: aktuelle_dateien,
            aktive_datei: None,
            status_meldung: String::from("Bereit"),
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    DateiAuswaehlen(PathBuf),
    Aktualisieren,
}

impl App {
    fn update(&mut self, message: Message) {
        match message {
            Message::DateiAuswaehlen(pfad) => {
                let name = pfad
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy();
                self.status_meldung = format!("Geladen: {name}");
                self.aktive_datei = Some(pfad);
            }
            Message::Aktualisieren => {
                let basis = PathBuf::from(".");
                self.dateien = file_tree::dateien_laden(&basis);
                self.status_meldung =
                    String::from("Dateiliste aktualisiert");
            }
        }
    }

    fn seitenleiste(&self) -> Element<'_, Message> {
        let ueberschrift = row![
            text("DATEIEN").size(14).color(theme::TEXT_GEDIKTET),
            Space::with_width(Length::Fill),
            button("Neu laden")
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
            Some(p) => p.display().to_string(),
            None => "Keine Datei geöffnet".to_string(),
        };

        let statistik_text = format!(
            "Status: {} | {} Zeichen | {} Wörter | Lesezeit: ~{} Min.",
            self.status_meldung,
            stats::zeichen_zaehlen(&self.status_meldung),
            stats::woerter_zaehlen(&self.status_meldung),
            stats::lesezeit_minuten(
                stats::woerter_zaehlen(&self.status_meldung)
            )
        );

        let haupt_bereich = container(
            column![
                text(datei_titel)
                    .size(24)
                    .color(theme::TEXT_HAUPT),
                text(statistik_text)
                    .size(13)
                    .color(theme::AKZENT_GOLD),
            ]
            .spacing(16),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(32)
        .style(|_| container::Style {
            background: Some(theme::HINTERGRUND.into()),
            ..container::Style::default()
        });

        row![seitenleiste, haupt_bereich].into()
    }
}
