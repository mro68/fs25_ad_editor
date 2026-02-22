//! Marker-Parsing: Konvertiert Marker-XML-Elemente in MapMarker-Objekte.

use anyhow::{bail, Context, Result};

/// Parst eine Marker-ID aus einem Text-Knoten.
///
/// AutoDrive speichert IDs als Gleitkommazahl, z.B. `"42.0"`. Die Funktion
/// akzeptiert ganze und gebrochene Zahlen, lehnt aber negative und nicht-endliche
/// Werte ab.
pub(super) fn parse_marker_id(text: &str) -> Result<u64> {
    let value = text
        .trim()
        .parse::<f64>()
        .context("Marker-ID ist keine gueltige Zahl")?;

    if !value.is_finite() {
        bail!("Marker-ID muss endlich sein");
    }

    if value < 0.0 {
        bail!("Marker-ID darf nicht negativ sein");
    }

    if value.fract() != 0.0 {
        bail!("Marker-ID muss ganzzahlig sein");
    }

    Ok(value as u64)
}
