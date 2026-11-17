//! Teil 1: Das erste Fenster
//! Die Rust-Werkstatt – Band 2: Native Desktop-Apps mit Iced
//! Autor: Volker Dähn

use iced::widget::{button, column, text};
use iced::Element;

fn main() -> iced::Result {
    iced::run("Die Rust-Werkstatt – Klick-Zähler", update, view)
}

#[derive(Default)]
struct Zustand {
    klicks: u32,
}

#[derive(Debug, Clone)]
enum Nachricht {
    Geklickt,
    Zuruecksetzen,
}

fn update(zustand: &mut Zustand, nachricht: Nachricht) {
    match nachricht {
        Nachricht::Geklickt => {
            zustand.klicks += 1;
        }
        Nachricht::Zuruecksetzen => {
            zustand.klicks = 0;
        }
    }
}

fn view(zustand: &Zustand) -> Element<'_, Nachricht> {
    let anzeige = text(format!("Anzahl Klicks: {}", zustand.klicks))
        .size(24);

    let knopf_plus = button("+1 Klick")
        .on_press(Nachricht::Geklickt);

    let knopf_reset = button("Zurücksetzen")
        .on_press(Nachricht::Zuruecksetzen);

    column![anzeige, knopf_plus, knopf_reset]
        .spacing(16)
        .padding(24)
        .into()
}
