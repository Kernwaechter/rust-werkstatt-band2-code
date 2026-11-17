use iced::time::every;
use iced::widget::canvas::{self, Frame, Geometry, Path, Stroke};
use iced::widget::{
    button, checkbox, column, container, row, text, Space,
};
use iced::{
    Alignment, Border, Color, Element, Length, Point, Rectangle,
    Renderer, Subscription, Task, Theme,
};
use std::collections::VecDeque;
use std::time::{Duration, Instant};

fn main() -> iced::Result {
    iced::application(
        "Ops-Wächter",
        OpsWaechter::update,
        OpsWaechter::view,
    )
    .subscription(OpsWaechter::subscription)
    .run()
}

struct LatenzGraph<'a> {
    verlauf: &'a VecDeque<u64>,
}

impl<'a> canvas::Program<Message> for LatenzGraph<'a> {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let w = bounds.width;
        let h = bounds.height;

        // 1. Hintergrund-Gitter zeichnen (alle 30 Pixel)
        let gitter_stift = Stroke::default()
            .with_color(Color::from_rgba(1.0, 1.0, 1.0, 0.06))
            .with_width(1.0);

        for y in (0..h as i32).step_by(30) {
            let linie = Path::line(
                Point::new(0.0, y as f32),
                Point::new(w, y as f32),
            );
            frame.stroke(&linie, gitter_stift);
        }

        // 2. Warn-Schwellenwert-Linie bei 250 ms (Gold/Gelb)
        let schwellen_y = h * 0.5; // Mitte entspricht ca. 250 ms
        let warn_stift = Stroke::default()
            .with_color(Color::from_rgba(0.95, 0.70, 0.10, 0.40))
            .with_width(1.0);
        let warn_linie = Path::line(
            Point::new(0.0, schwellen_y),
            Point::new(w, schwellen_y),
        );
        frame.stroke(&warn_linie, warn_stift);

        // 3. Latenzkurve zeichnen
        if self.verlauf.len() >= 2 {
            let max_skala = 500.0_f32; // 500 ms Maximalhöhe
            let punkte_anzahl = (self.verlauf.len() - 1) as f32;
            let schritt_x = w / punkte_anzahl.max(1.0);

            let mut pfad_bauer =
                iced::widget::canvas::path::Builder::new();

            for (i, &latenz) in self.verlauf.iter().enumerate() {
                let x = i as f32 * schritt_x;
                let bruch = latenz as f32 / max_skala;
                let anteil = bruch.clamp(0.0, 1.0);
                let y = h - (anteil * (h - 8.0)) - 4.0;

                let punkt = Point::new(x, y);
                if i == 0 {
                    pfad_bauer.move_to(punkt);
                } else {
                    pfad_bauer.line_to(punkt);
                }
            }

            let kurve = pfad_bauer.build();
            let linien_stift = Stroke::default()
                .with_color(Color::from_rgb(0.20, 0.85, 0.35))
                .with_width(2.5);

            frame.stroke(&kurve, linien_stift);
        }

        vec![frame.into_geometry()]
    }
}

struct OpsWaechter {
    latenz_verlauf: VecDeque<u64>,
    aktuelle_latenz: u64,
    auto_check: bool,
}

impl Default for OpsWaechter {
    fn default() -> Self {
        let mut v = VecDeque::new();
        v.push_back(42);
        v.push_back(38);
        v.push_back(55);
        v.push_back(48);

        Self {
            latenz_verlauf: v,
            aktuelle_latenz: 48,
            auto_check: true,
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    Pruefen,
    MessungDa(Result<u64, String>),
    AutoCheckUmschalten(bool),
    Tick,
}

async fn check_latenz() -> Result<u64, String> {
    let start = Instant::now();
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(3))
        .build();

    match client {
        Ok(c) => match c.get("https://rust-werkstatt.de").send().await {
            Ok(_) => Ok(start.elapsed().as_millis() as u64),
            Err(e) => Err(e.to_string()),
        },
        Err(e) => Err(e.to_string()),
    }
}

impl OpsWaechter {
    fn subscription(&self) -> Subscription<Message> {
        if self.auto_check {
            every(Duration::from_secs(5)).map(|_| Message::Tick)
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
            Message::Tick | Message::Pruefen => {
                Task::perform(check_latenz(), Message::MessungDa)
            }
            Message::MessungDa(ergebnis) => {
                let gemessen = ergebnis.unwrap_or(500);
                self.aktuelle_latenz = gemessen;

                self.latenz_verlauf.push_back(gemessen);
                if self.latenz_verlauf.len() > 30 {
                    self.latenz_verlauf.pop_front();
                }

                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let titel = text("OPS-WÄCHTER – LIVE GRAFIK").size(22);

        let kopf = row![
            titel,
            Space::with_width(Length::Fill),
            text(format!("{} ms", self.aktuelle_latenz))
                .size(16)
                .color(Color::from_rgb(0.96, 0.62, 0.07)),
            Space::with_width(16),
            checkbox("Live-Ticker (5s)", self.auto_check)
                .on_toggle(Message::AutoCheckUmschalten),
            button("Jetzt messen")
                .on_press(Message::Pruefen)
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

        let graph_leinwand = canvas::Canvas::new(LatenzGraph {
            verlauf: &self.latenz_verlauf,
        })
        .width(Length::Fill)
        .height(180);

        let graph_box = container(graph_leinwand)
            .width(Length::Fill)
            .padding(16)
            .style(|_| container::Style {
                background: Some(
                    Color::from_rgb(0.06, 0.08, 0.11).into(),
                ),
                border: Border {
                    color: Color::from_rgb(0.20, 0.25, 0.32),
                    width: 1.0,
                    radius: 8.0.into(),
                },
                ..container::Style::default()
            });

        let inhalt = column![
            kopf,
            graph_box,
            text("Skala: 0 bis 500 ms (Gelbe Linie = 250 ms)")
                .size(12)
                .color(Color::from_rgb(0.60, 0.65, 0.72)),
        ]
        .spacing(16);

        container(inhalt)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(24)
            .style(|_| container::Style {
                background: Some(
                    Color::from_rgb(0.09, 0.11, 0.15).into(),
                ),
                ..container::Style::default()
            })
            .into()
    }
}
