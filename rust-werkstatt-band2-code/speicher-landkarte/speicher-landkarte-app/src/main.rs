use std::path::PathBuf;

use iced::mouse;
use iced::widget::canvas::{self, Canvas, Frame, Geometry, Path, Stroke, Text};
use iced::widget::{
    button, column, container, horizontal_rule, horizontal_space, row, scrollable, text,
    text_input,
};
use iced::{Alignment, Color, Element, Length, Point, Rectangle, Renderer, Size, Task, Theme};

use speicher_landkarte_core::{
    build_artefakte_finden, format_groesse, ordner_bereinigen, pfad_suchen, treemap_berechnen,
    verzeichnis_scannen, Bereich, Kachel, Kategorie, Knoten,
};

pub fn main() -> iced::Result {
    iced::application(
        "Speicher-Landkarte — Rust-Werkstatt",
        App::update,
        App::view,
    )
    .theme(App::theme)
    .window_size(Size::new(1240.0, 800.0))
    .run_with(App::new)
}

#[derive(Debug, Clone)]
enum Message {
    PfadGeaendert(String),
    ScanStarten,
    ScanBeendet(Result<Knoten, String>),
    ThemeUmschalten,
    ZoomAufPfad(PathBuf),
    KachelHover(Option<usize>),
    KachelGeklickt(usize),
    LoeschenAnfragen(PathBuf, u64),
    LoeschenBestaetigen,
    LoeschenAbbrechen,
    Bereinigt(Result<(PathBuf, u64), String>),
}

#[derive(Debug, Clone)]
enum ScanStatus {
    Bereit,
    Laeuft,
    Fehler,
}

struct App {
    pfad_eingabe: String,
    wurzel: Option<Knoten>,
    aktueller_pfad: Option<PathBuf>,
    hover_index: Option<usize>,
    status: ScanStatus,
    hinweis: String,
    dark_mode: bool,
    loesch_kandidat: Option<(PathBuf, u64)>,
    canvas_cache: canvas::Cache,
}

impl App {
    fn new() -> (Self, Task<Message>) {
        let start_pfad = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .display()
            .to_string();

        let app = Self {
            pfad_eingabe: start_pfad.clone(),
            wurzel: None,
            aktueller_pfad: None,
            hover_index: None,
            status: ScanStatus::Laeuft,
            hinweis: String::from("Initialer Scan läuft …"),
            dark_mode: true,
            loesch_kandidat: None,
            canvas_cache: canvas::Cache::new(),
        };

        let task = Task::perform(scan_async(PathBuf::from(start_pfad)), Message::ScanBeendet);
        (app, task)
    }

    fn theme(&self) -> Theme {
        if self.dark_mode {
            Theme::Dark
        } else {
            Theme::Light
        }
    }

    fn aktueller_knoten(&self) -> Option<&Knoten> {
        let wurzel = self.wurzel.as_ref()?;
        match &self.aktueller_pfad {
            Some(ziel) => pfad_suchen(wurzel, ziel).or(Some(wurzel)),
            None => Some(wurzel),
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::PfadGeaendert(neu) => {
                self.pfad_eingabe = neu;
                Task::none()
            }
            Message::ScanStarten => {
                let pfad = PathBuf::from(self.pfad_eingabe.trim());
                self.status = ScanStatus::Laeuft;
                self.hinweis = format!("Scanne {} …", pfad.display());
                self.loesch_kandidat = None;
                self.hover_index = None;
                Task::perform(scan_async(pfad), Message::ScanBeendet)
            }
            Message::ScanBeendet(Ok(knoten)) => {
                self.aktueller_pfad = Some(knoten.pfad.clone());
                self.hinweis = format!(
                    "Scan abgeschlossen: {} insgesamt",
                    format_groesse(knoten.groesse)
                );
                self.wurzel = Some(knoten);
                self.status = ScanStatus::Bereit;
                self.hover_index = None;
                self.canvas_cache.clear();
                Task::none()
            }
            Message::ScanBeendet(Err(fehler)) => {
                self.status = ScanStatus::Fehler;
                self.hinweis = format!("Fehler beim Scannen: {fehler}");
                Task::none()
            }
            Message::ThemeUmschalten => {
                self.dark_mode = !self.dark_mode;
                self.canvas_cache.clear();
                Task::none()
            }
            Message::ZoomAufPfad(ziel) => {
                self.aktueller_pfad = Some(ziel);
                self.hover_index = None;
                self.canvas_cache.clear();
                Task::none()
            }
            Message::KachelHover(index) => {
                if self.hover_index != index {
                    self.hover_index = index;
                    self.canvas_cache.clear();
                }
                Task::none()
            }
            Message::KachelGeklickt(index) => {
                if let Some(knoten) = self.aktueller_knoten() {
                    if let Some(kind) = knoten.kinder.get(index) {
                        if kind.ist_ordner && !kind.kinder.is_empty() {
                            self.aktueller_pfad = Some(kind.pfad.clone());
                            self.hover_index = None;
                            self.canvas_cache.clear();
                        }
                    }
                }
                Task::none()
            }
            Message::LoeschenAnfragen(pfad, groesse) => {
                self.loesch_kandidat = Some((pfad, groesse));
                Task::none()
            }
            Message::LoeschenAbbrechen => {
                self.loesch_kandidat = None;
                Task::none()
            }
            Message::LoeschenBestaetigen => {
                if let Some((pfad, groesse)) = self.loesch_kandidat.take() {
                    self.hinweis = format!("Bereinige {} …", pfad.display());
                    return Task::perform(
                        async move {
                            let ergebnis = tokio::task::spawn_blocking(move || {
                                ordner_bereinigen(&pfad, groesse, false)
                                    .map(|bytes| (pfad, bytes))
                                    .map_err(|e| e.to_string())
                            })
                            .await
                            .map_err(|e| e.to_string())?;
                            ergebnis
                        },
                        Message::Bereinigt,
                    );
                }
                Task::none()
            }
            Message::Bereinigt(Ok((pfad, bytes))) => {
                self.hinweis = format!(
                    "Bereinigt: {} ({} freigegeben)",
                    pfad.display(),
                    format_groesse(bytes)
                );
                let neu_scan = PathBuf::from(self.pfad_eingabe.trim());
                self.status = ScanStatus::Laeuft;
                Task::perform(scan_async(neu_scan), Message::ScanBeendet)
            }
            Message::Bereinigt(Err(fehler)) => {
                self.hinweis = format!("Bereinigung fehlgeschlagen: {fehler}");
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let kopfleiste = row![
            text("Speicher-Landkarte").size(20),
            text_input("Projektpfad eingeben …", &self.pfad_eingabe)
                .on_input(Message::PfadGeaendert)
                .on_submit(Message::ScanStarten)
                .width(Length::Fill),
            button(match self.status {
                ScanStatus::Laeuft => "Scanne …",
                _ => "Scannen",
            })
            .on_press(Message::ScanStarten),
            button(if self.dark_mode {
                "Light Mode"
            } else {
                "Dark Mode"
            })
            .on_press(Message::ThemeUmschalten),
        ]
        .spacing(12)
        .align_y(Alignment::Center);

        let breadcrumbs = self.view_breadcrumbs();
        let hauptbereich = self.view_hauptbereich();
        let statusleiste = self.view_statusleiste();

        let gesamt = column![
            kopfleiste,
            breadcrumbs,
            horizontal_rule(1),
            hauptbereich,
            horizontal_rule(1),
            statusleiste,
        ]
        .spacing(10)
        .padding(16);

        container(gesamt)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn view_breadcrumbs(&self) -> Element<'_, Message> {
        let Some(wurzel) = &self.wurzel else {
            return row![text("Noch kein Verzeichnis geladen.").size(13)].into();
        };

        let aktuell = self
            .aktueller_pfad
            .as_deref()
            .unwrap_or(wurzel.pfad.as_path());

        let mut schritte: Vec<Element<'_, Message>> = vec![button(
            text(format!("Wurzel: {}", wurzel.name)).size(13),
        )
        .on_press(Message::ZoomAufPfad(wurzel.pfad.clone()))
        .into()];

        if let Ok(rel) = aktuell.strip_prefix(&wurzel.pfad) {
            let mut laufend = wurzel.pfad.clone();
            for teil in rel.components() {
                laufend.push(teil.as_os_str());
                let label = teil.as_os_str().to_string_lossy().into_owned();
                let ziel = laufend.clone();
                schritte.push(text("›").size(14).into());
                schritte.push(
                    button(text(label).size(13))
                        .on_press(Message::ZoomAufPfad(ziel))
                        .into(),
                );
            }
        }

        row(schritte).spacing(6).align_y(Alignment::Center).into()
    }

    fn view_hauptbereich(&self) -> Element<'_, Message> {
        let Some(knoten) = self.aktueller_knoten() else {
            return container(
                text("Starte einen Scan, um die Speicher-Landkarte aufzubauen.").size(15),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into();
        };

        let treemap_widget: Element<'_, Message> = Canvas::new(TreemapCanvas {
            knoten: knoten.clone(),
            hover_index: self.hover_index,
            dark_mode: self.dark_mode,
            cache: &self.canvas_cache,
        })
        .width(Length::Fill)
        .height(Length::Fill)
        .into();

        let inspektor = self.view_inspektor(knoten);

        row![
            container(treemap_widget)
                .width(Length::FillPortion(3))
                .height(Length::Fill),
            container(inspektor)
                .width(Length::Fixed(300.0))
                .height(Length::Fill)
                .padding(12),
        ]
        .spacing(16)
        .height(Length::Fill)
        .into()
    }

    fn view_inspektor<'a>(&'a self, knoten: &'a Knoten) -> Element<'a, Message> {
        let hover_info: Element<'a, Message> = if let Some(idx) = self.hover_index {
            if let Some(kind) = knoten.kinder.get(idx) {
                column![
                    text("Ausgewählte Kachel").size(15),
                    text(&kind.name).size(14),
                    text(format!(
                        "{} · {}",
                        format_groesse(kind.groesse),
                        kind.kategorie.bezeichnung()
                    ))
                    .size(13),
                    text(if kind.ist_ordner {
                        "Klick: In Ordner hineinzoomen"
                    } else {
                        "Datei (Endknoten)"
                    })
                    .size(12),
                ]
                .spacing(4)
                .into()
            } else {
                text("Bewege die Maus über eine Kachel.").size(13).into()
            }
        } else {
            column![
                text("Aktuelle Ansicht").size(15),
                text(&knoten.name).size(14),
                text(format!("Größe: {}", format_groesse(knoten.groesse))).size(13),
                text("Klick auf Ordner-Kachel zoomt hinein.").size(12),
            ]
            .spacing(4)
            .into()
        };

        let legende = column![
            text("Kategorien (Farbe + Text)").size(14),
            legende_zeile("Build-Artefakt", Kategorie::BuildArtefakt, self.dark_mode),
            legende_zeile("Quellcode", Kategorie::Quellcode, self.dark_mode),
            legende_zeile("Medien", Kategorie::Medien, self.dark_mode),
            legende_zeile("Ordner", Kategorie::Ordner, self.dark_mode),
            legende_zeile("Sonstiges", Kategorie::Sonstiges, self.dark_mode),
        ]
        .spacing(6);

        let artefakte = build_artefakte_finden(knoten);
        let mut artefakt_liste = column![text(format!(
            "Build-Artefakte ({})",
            artefakte.len()
        ))
        .size(14)]
        .spacing(8);

        for art in artefakte.into_iter().take(6) {
            let pfad = art.pfad.clone();
            let groesse = art.groesse;
            artefakt_liste = artefakt_liste.push(
                row![
                    column![
                        text(&art.name).size(13),
                        text(format_groesse(art.groesse)).size(12),
                    ]
                    .width(Length::Fill),
                    button(text("Aufräumen").size(12))
                        .on_press(Message::LoeschenAnfragen(pfad, groesse)),
                ]
                .align_y(Alignment::Center)
                .spacing(8),
            );
        }

        let bestaetigung: Element<'a, Message> =
            if let Some((pfad, groesse)) = &self.loesch_kandidat {
                column![
                    horizontal_rule(1),
                    text("Sicherheitsabfrage").size(14),
                    text(format!(
                        "{} ({}) endgültig löschen?",
                        pfad.file_name()
                            .map(|n| n.to_string_lossy().into_owned())
                            .unwrap_or_else(|| pfad.display().to_string()),
                        format_groesse(*groesse)
                    ))
                    .size(12),
                    row![
                        button(text("Löschen").size(12))
                            .on_press(Message::LoeschenBestaetigen),
                        button(text("Abbrechen").size(12))
                            .on_press(Message::LoeschenAbbrechen),
                    ]
                    .spacing(8),
                ]
                .spacing(6)
                .into()
            } else {
                column![].into()
            };

        scrollable(
            column![
                hover_info,
                horizontal_rule(1),
                legende,
                horizontal_rule(1),
                artefakt_liste,
                bestaetigung,
            ]
            .spacing(12),
        )
        .into()
    }

    fn view_statusleiste(&self) -> Element<'_, Message> {
        row![
            text(&self.hinweis).size(13),
            horizontal_space(),
            text("Maus: Details · Klick: Zoom").size(12),
        ]
        .align_y(Alignment::Center)
        .into()
    }
}

fn legende_zeile<'a>(label: &'a str, kategorie: Kategorie, dark_mode: bool) -> Element<'a, Message> {
    let farbe = kategorie_farbe(kategorie, dark_mode);
    row![
        container(text(" ").size(12))
            .width(Length::Fixed(14.0))
            .height(Length::Fixed(14.0))
            .style(move |_| container::Style {
                background: Some(farbe.into()),
                ..Default::default()
            }),
        text(label).size(13),
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .into()
}

/// Wahrnehmungsfreundliche, farbenblind-sichere Palette (Schiefergrau-kompatibel).
fn kategorie_farbe(kategorie: Kategorie, dark_mode: bool) -> Color {
    match (kategorie, dark_mode) {
        (Kategorie::BuildArtefakt, true) => Color::from_rgb8(217, 119, 6),
        (Kategorie::BuildArtefakt, false) => Color::from_rgb8(245, 158, 11),
        (Kategorie::Quellcode, true) => Color::from_rgb8(37, 99, 235),
        (Kategorie::Quellcode, false) => Color::from_rgb8(59, 130, 246),
        (Kategorie::Medien, true) => Color::from_rgb8(13, 148, 136),
        (Kategorie::Medien, false) => Color::from_rgb8(20, 184, 166),
        (Kategorie::Ordner, true) => Color::from_rgb8(79, 70, 229),
        (Kategorie::Ordner, false) => Color::from_rgb8(99, 102, 241),
        (Kategorie::Sonstiges, true) => Color::from_rgb8(71, 85, 105),
        (Kategorie::Sonstiges, false) => Color::from_rgb8(148, 163, 184),
    }
}

async fn scan_async(pfad: PathBuf) -> Result<Knoten, String> {
    tokio::task::spawn_blocking(move || verzeichnis_scannen(&pfad).map_err(|e| e.to_string()))
        .await
        .map_err(|e| e.to_string())?
}

struct TreemapCanvas<'a> {
    knoten: Knoten,
    hover_index: Option<usize>,
    dark_mode: bool,
    cache: &'a canvas::Cache,
}

impl<'a> canvas::Program<Message> for TreemapCanvas<'a> {
    type State = Vec<Kachel>;

    fn update(
        &self,
        state: &mut Self::State,
        event: canvas::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> (canvas::event::Status, Option<Message>) {
        let flaeche = Bereich {
            x: 0.0,
            y: 0.0,
            breite: bounds.width,
            hoehe: bounds.height,
        };
        *state = treemap_berechnen(&self.knoten, flaeche);

        let Some(pos) = cursor.position_in(bounds) else {
            if self.hover_index.is_some() {
                return (canvas::event::Status::Ignored, Some(Message::KachelHover(None)));
            }
            return (canvas::event::Status::Ignored, None);
        };

        let treffer = state
            .iter()
            .position(|k| k.bereich.enthaelt(pos.x, pos.y));

        match event {
            canvas::Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if treffer != self.hover_index {
                    return (
                        canvas::event::Status::Captured,
                        Some(Message::KachelHover(treffer)),
                    );
                }
            }
            canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(idx) = treffer {
                    return (
                        canvas::event::Status::Captured,
                        Some(Message::KachelGeklickt(idx)),
                    );
                }
            }
            _ => {}
        }

        (canvas::event::Status::Ignored, None)
    }

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let geom = self.cache.draw(renderer, bounds.size(), |frame: &mut Frame| {
            let hintergrund = if self.dark_mode {
                Color::from_rgb8(24, 27, 34)
            } else {
                Color::from_rgb8(241, 245, 249)
            };
            frame.fill_rectangle(Point::ORIGIN, bounds.size(), hintergrund);

            let flaeche = Bereich {
                x: 4.0,
                y: 4.0,
                breite: (bounds.width - 8.0).max(0.0),
                hoehe: (bounds.height - 8.0).max(0.0),
            };
            let kacheln = treemap_berechnen(&self.knoten, flaeche);

            for (idx, k) in kacheln.iter().enumerate() {
                let b = k.bereich;
                let farbe = kategorie_farbe(k.kategorie, self.dark_mode);
                let ist_hover = self.hover_index == Some(idx);

                let rechteck = Path::rectangle(Point::new(b.x, b.y), Size::new(b.breite, b.hoehe));
                frame.fill(&rechteck, farbe);

                let rahmen_farbe = if ist_hover {
                    Color::WHITE
                } else if self.dark_mode {
                    Color::from_rgb8(17, 20, 26)
                } else {
                    Color::from_rgb8(255, 255, 255)
                };
                frame.stroke(
                    &rechteck,
                    Stroke::default()
                        .with_width(if ist_hover { 2.5 } else { 1.0 })
                        .with_color(rahmen_farbe),
                );

                // Progressive Disclosure: Beschriftung nur zeichnen, wenn genug Platz ist.
                if b.breite > 85.0 && b.hoehe > 36.0 {
                    let titel = if k.kategorie == Kategorie::BuildArtefakt {
                        format!("[Build] {}", k.name)
                    } else {
                        k.name.clone()
                    };
                    frame.fill_text(Text {
                        content: titel,
                        position: Point::new(b.x + 6.0, b.y + 6.0),
                        color: Color::WHITE,
                        size: 13.0.into(),
                        ..Text::default()
                    });
                    if b.hoehe > 54.0 {
                        frame.fill_text(Text {
                            content: format!("{} ({:.0} %)", format_groesse(k.groesse), k.anteil),
                            position: Point::new(b.x + 6.0, b.y + 24.0),
                            color: Color::from_rgba(1.0, 1.0, 1.0, 0.85),
                            size: 11.0.into(),
                            ..Text::default()
                        });
                    }
                }
            }
        });

        vec![geom]
    }
}
