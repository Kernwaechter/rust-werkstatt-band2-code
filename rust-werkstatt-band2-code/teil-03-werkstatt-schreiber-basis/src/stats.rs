pub(crate) fn woerter_zaehlen(text: &str) -> usize {
    text.split_whitespace().count()
}

pub(crate) fn zeichen_zaehlen(text: &str) -> usize {
    text.chars()
        .filter(|c| *c != '\r' && *c != '\n')
        .count()
}

pub(crate) fn lesezeit_minuten(woerter: usize) -> usize {
    // Durchschnittliche Lesegeschwindigkeit: ca. 200 Wörter / Min
    woerter.div_ceil(200)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_woerter_zaehlen() {
        assert_eq!(woerter_zaehlen("Hallo Werkstatt"), 2);
        assert_eq!(woerter_zaehlen("   "), 0);
    }

    #[test]
    fn test_zeichen_zaehlen() {
        assert_eq!(zeichen_zaehlen("Rust\n"), 4);
    }

    #[test]
    fn test_lesezeit() {
        assert_eq!(lesezeit_minuten(50), 1);
        assert_eq!(lesezeit_minuten(250), 2);
    }
}
