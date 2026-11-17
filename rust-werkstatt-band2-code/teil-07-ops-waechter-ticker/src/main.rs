use iced::time::every;
use iced::widget::{button, checkbox, column, container, row, text, Space};
use iced::{Alignment, Border, Color, Element, Length, Subscription, Task};
use std::time::{Duration, Instant};

fn main() -> iced::Result {
    iced::application(
        "Ops-Wächter – Live Dashboard",
        OpsWaechter::update,
        OpsWaechter::view,
    )
    .subscription(OpsWaechter::subscription)
    .run()
}

#[derive(Debug, Clone)]
struct ServerZiel {
    name: String,
    url: String,
    ist_online: bool,
    latenz_ms: u64,
    http_code: u16,
    letzte_meldung: String,
}

struct OpsWaechter {
    server_liste: Vec<ServerZiel>,
    auto_check: bool,
    takt_sekunden: u64,
}

impl Default for OpsWaechter {
    fn default() -> Self {
        Self {
            server_liste: vec![
                ServerZiel {
                    name: "Rust-Werkstatt".into(),
                    url: "https://rust-werkstatt.de".into(),
                    ist_online: false,
                    latenz_ms: 0,
                    http_code: 0,
                    letzte_meldung: "Wartet auf Start".into(),
                },
                ServerZiel {
                    name: "Crates.io Paketarchiv".into(),
                    url: "https://crates.io".into(),
                    ist_online: false,
                    latenz_ms: 0,
                    http_code: 0,
                    letzte_meldung: "Wartet auf Start".into(),
                },
                ServerZiel {
                    name: "Lokaler Test-Dienst".into(),
                    url: "http://127.0.0.1:8080".into(),
                    ist_online: false,
                    latenz_ms: 0,
                    http_code: 0,
                    letzte_meldung: "Wartet auf Start".into(),
                },
            ],
            auto_check: true,
            takt_sekunden: 10,
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    AllePruefen,
    AutoCheckUmschalten(bool),
    ErgebnisEmpfangen(usize, Result<(u16, u64), String>),
    Tick,
}

async fn einzel_check(
    index: usize,
    url: String,
) -> (usize, Result<(u16, u64), String>) {
    let start = Instant::now();
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(3))
        .build();

    match client {
        Ok(c) => match c.get(&url).send().await {
            Ok(resp) => {
                let latenz = start.elapsed().as_millis() as u64;
                let code = resp.status().as_u16();
                (index, Ok((code, latenz)))
            }
            Err(err) => (index, Err(err.to_string())),
        },
        Err(err) => (index, Err(err.to_string())),
    }
}

impl OpsWaechter {
    fn subscription(&self) -> Subscription<Message> {
        if self.auto_check {
            every(Duration::from_secs(self.takt_sekunden))
                .map(|_| Message::Tick)
        } else {
            Subscription::none()
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::AutoCheckUmschalten(aktiv) => {
                self.auto_check = aktiv;
                Task::none()
            }
            Message::Tick | Message::AllePruefen => {
                let mut aufgaben = Vec::new();

                for (idx, ziel) in self.server_liste.iter_mut().enumerate() {
                    ziel.letzte_meldung = String::from("Prüfe...");
                    let url_kopie = ziel.url.clone();
                    aufgaben.push(Task::perform(
                        einzel_check(idx, url_kopie),
                        |(i, res)| Message::ErgebnisEmpfangen(i, res),
                    ));
                }

                Task::batch(aufgaben)
            }
            Message::ErgebnisEmpfangen(idx, ergebnis) => {
                if let Some(ziel) = self.server_liste.get_mut(idx) {
                    match ergebnis {
                        Ok((code, latenz)) => {
                            ziel.http_code = code;
                            ziel.latenz_ms = latenz;
                            let ok_bereich = 200..=399;
                            let ist_ok = ok_bereich.contains(&code);
                            ziel.ist_online = ist_ok;
                            ziel.letzte_meldung = format!("{code} OK");
                        }
                        Err(err) => {
                            ziel.http_code = 0;
                            ziel.latenz_ms = 0;
                            ziel.ist_online = false;
                            ziel.letzte_meldung = if err.contains("timed out") {
                                "Timeout (3s)".into()
                            } else {
                                "Verbindungsfehler".into()
                            };
                        }
                    }
                }
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let titel = text("OPS-WÄCHTER – LIVE DASHBOARD").size(22);

        let kopf_leiste = row![
            titel,
            Space::with_width(Length::Fill),
            checkbox("Auto-Refresh (10s)", self.auto_check)
                .on_toggle(Message::AutoCheckUmschalten),
            button("Jetzt alle prüfen")
                .on_press(Message::AllePruefen)
                .padding([8, 16])
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
                }),
        ]
        .spacing(16)
        .align_y(Alignment::Center);

        let mut karten_spalte = column![].spacing(12);

        for ziel in &self.server_liste {
            let status_badge = if ziel.letzte_meldung == "Prüfe..." {
                text("◌ PRÜFE")
                    .size(13)
                    .color(Color::from_rgb(0.95, 0.70, 0.10))
            } else if ziel.ist_online {
                text("● ONLINE")
                    .size(13)
                    .color(Color::from_rgb(0.20, 0.85, 0.35))
            } else {
                text("■ OFFLINE")
                    .size(13)
                    .color(Color::from_rgb(0.92, 0.25, 0.25))
            };

            let zeile = row![
                status_badge.width(90),
                column![
                    text(&ziel.name).size(15),
                    text(&ziel.url)
                        .size(12)
                        .color(Color::from_rgb(0.55, 0.60, 0.70)),
                ]
                .width(220),
                Space::with_width(Length::Fill),
                text(&ziel.letzte_meldung).size(14),
                Space::with_width(20),
                text(if ziel.latenz_ms > 0 {
                    format!("{} ms", ziel.latenz_ms)
                } else {
                    "--".into()
                })
                .size(14),
            ]
            .align_y(Alignment::Center)
            .padding(16);

            let karte = container(zeile)
                .width(Length::Fill)
                .style(|_| container::Style {
                    background: Some(Color::from_rgb(0.12, 0.15, 0.20).into()),
                    border: Border {
                        color: Color::from_rgb(0.22, 0.27, 0.35),
                        width: 1.0,
                        radius: 8.0.into(),
                    },
                    ..container::Style::default()
                });

            karten_spalte = karten_spalte.push(karte);
        }

        let inhalt = column![kopf_leiste, karten_spalte].spacing(24);

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
}
