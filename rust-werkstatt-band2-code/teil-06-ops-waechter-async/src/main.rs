use iced::widget::{button, column, container, row, text, Space};
use iced::{Alignment, Border, Color, Element, Length, Task};
use std::time::Instant;

fn main() -> iced::Result {
    iced::application(
        "Ops-Wächter",
        OpsWaechter::update,
        OpsWaechter::view,
    )
    .run()
}

#[derive(Debug, Clone)]
struct ServerStatus {
    url: String,
    ist_online: bool,
    latenz_ms: u64,
    http_code: u16,
    letzte_pruefung: String,
}

struct OpsWaechter {
    server: ServerStatus,
    wird_geprueft: bool,
}

impl Default for OpsWaechter {
    fn default() -> Self {
        Self {
            server: ServerStatus {
                url: String::from("https://rust-werkstatt.de"),
                ist_online: false,
                latenz_ms: 0,
                http_code: 0,
                letzte_pruefung: String::from("Noch nicht geprüft"),
            },
            wird_geprueft: false,
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    JetztPruefen,
    PruefungFertig(Result<(u16, u64), String>),
}

async fn netzwerk_check(url: String) -> Result<(u16, u64), String> {
    let start = Instant::now();
    let antwort = reqwest::get(&url)
        .await
        .map_err(|e| e.to_string())?;

    let latenz = start.elapsed().as_millis() as u64;
    let code = antwort.status().as_u16();

    Ok((code, latenz))
}

impl OpsWaechter {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::JetztPruefen => {
                self.wird_geprueft = true;
                let ziel_url = self.server.url.clone();
                Task::perform(
                    netzwerk_check(ziel_url),
                    Message::PruefungFertig,
                )
            }
            Message::PruefungFertig(ergebnis) => {
                self.wird_geprueft = false;
                match ergebnis {
                    Ok((code, latenz)) => {
                        let ist_ok = (200..=399).contains(&code);
                        self.server.ist_online = ist_ok;
                        self.server.http_code = code;
                        self.server.latenz_ms = latenz;
                        self.server.letzte_pruefung =
                            String::from("Gerade eben");
                    }
                    Err(_) => {
                        self.server.ist_online = false;
                        self.server.http_code = 0;
                        self.server.latenz_ms = 0;
                        self.server.letzte_pruefung =
                            String::from("Fehler beim Verbinden");
                    }
                }
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let titel = text("OPS-WÄCHTER")
            .size(24);

        let status_kugel = if self.wird_geprueft {
            text("◌ PRÜFE...").color(Color::from_rgb(0.95, 0.70, 0.10))
        } else if self.server.ist_online {
            text("● ONLINE").color(Color::from_rgb(0.20, 0.85, 0.35))
        } else {
            text("■ OFFLINE").color(Color::from_rgb(0.92, 0.25, 0.25))
        };

        let server_zeile = row![
            status_kugel.size(15),
            text(&self.server.url).size(15),
            Space::with_width(Length::Fill),
            text(format!("HTTP {}", self.server.http_code)).size(13),
            text(format!("{} ms", self.server.latenz_ms)).size(13),
        ]
        .spacing(16)
        .align_y(Alignment::Center);

        let karten_stil = container(server_zeile)
            .width(Length::Fill)
            .padding(16)
            .style(|_| container::Style {
                background: Some(
                    Color::from_rgb(0.12, 0.15, 0.20).into(),
                ),
                border: Border {
                    color: Color::from_rgb(0.25, 0.30, 0.38),
                    width: 1.0,
                    radius: 8.0.into(),
                },
                ..container::Style::default()
            });

        let knopf_check = if self.wird_geprueft {
            button("Prüfung läuft...")
                .padding([10, 20])
        } else {
            button("Jetzt prüfen")
                .on_press(Message::JetztPruefen)
                .padding([10, 20])
                .style(|_, status| {
                    let mut stil = button::Style::default();
                    if status == button::Status::Hovered {
                        stil.background = Some(Color::from_rgb(1.0, 0.45, 0.15).into());
                    } else if status == button::Status::Pressed {
                        stil.background = Some(Color::from_rgb(0.75, 0.25, 0.02).into());
                    } else {
                        stil.background = Some(Color::from_rgb(0.92, 0.35, 0.05).into());
                    }
                    stil
                })
        };

        let inhalt = column![
            titel,
            karten_stil,
            text(&self.server.letzte_pruefung)
                .size(13)
                .color(Color::from_rgb(0.60, 0.65, 0.72)),
            knopf_check,
        ]
        .spacing(16);

        container(inhalt)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(24)
            .style(|_| container::Style {
                background: Some(
                    Color::from_rgb(0.06, 0.08, 0.11).into(),
                ),
                ..container::Style::default()
            })
            .into()
    }
}
