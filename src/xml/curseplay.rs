//! Curseplay XML-Import/Export fuer Feldumrandungen.
//!
//! Parst und schreibt das Curseplay `<customField>`-Format, das Vertex-Positionen
//! als `x z`-Paare (Leerzeichen-getrennt) in `<vertex>`-Tags speichert.
//! Der letzte Vertex ist identisch mit dem ersten (Ring-Marker) und wird beim
//! Import automatisch entfernt.

use anyhow::{anyhow, Context, Result};
use glam::Vec2;
use quick_xml::events::Event;
use quick_xml::Reader;

/// Parst eine Curseplay-XML-Datei und gibt die Vertex-Positionen zurueck.
///
/// Der ringschliessende letzte Vertex (identisch mit dem ersten) wird automatisch
/// entfernt. Gibt einen Fehler zurueck wenn das XML ungueltig ist oder keine
/// Vertices enthaelt.
pub fn parse_curseplay(xml_content: &str) -> Result<Vec<Vec2>> {
    let mut reader = Reader::from_str(xml_content);
    reader.config_mut().trim_text(true);

    let mut vertices: Vec<Vec2> = Vec::new();
    let mut in_vertex = false;

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"vertex" => {
                in_vertex = true;
            }
            Ok(Event::Empty(ref e)) if e.name().as_ref() == b"vertex" => {
                // Leere <vertex/>-Tags haben keinen Text-Content – uebergehen
                in_vertex = false;
            }
            Ok(Event::Text(ref e)) if in_vertex => {
                let text = e
                    .xml_content()
                    .context("Vertex-Text ungueltig")?
                    .into_owned();
                let text = text.trim();
                let mut parts = text.split_whitespace();
                let x_str = parts
                    .next()
                    .ok_or_else(|| anyhow!("Vertex hat kein x: '{}'", text))?;
                let z_str = parts
                    .next()
                    .ok_or_else(|| anyhow!("Vertex hat kein z: '{}'", text))?;
                let x: f32 = x_str
                    .parse()
                    .with_context(|| format!("Ungueltige x-Koordinate: '{}'", x_str))?;
                let z: f32 = z_str
                    .parse()
                    .with_context(|| format!("Ungueltige z-Koordinate: '{}'", z_str))?;
                vertices.push(Vec2::new(x, z));
            }
            Ok(Event::End(ref e)) if e.name().as_ref() == b"vertex" => {
                in_vertex = false;
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(anyhow!("XML-Lesefehler: {}", e)),
            _ => {}
        }
    }

    if vertices.is_empty() {
        return Err(anyhow!("Keine Vertices gefunden"));
    }

    // Ringschluss entfernen: letzter Vertex == erster?
    if vertices.len() >= 2 {
        let first = vertices[0];
        let last = *vertices.last().expect("nicht leer");
        if (first - last).length_squared() < 1e-6 {
            vertices.pop();
        }
    }

    if vertices.is_empty() {
        return Err(anyhow!("Keine verwertbaren Vertices nach Ring-Bereinigung"));
    }

    Ok(vertices)
}

/// Schreibt eine Liste von Positionen als Curseplay XML.
///
/// Der erste Vertex wird am Ende wiederholt (Ring-Marker).
pub fn write_curseplay(vertices: &[Vec2]) -> String {
    let mut out = String::from("<?xml version=\"1.0\" encoding=\"utf-8\" standalone=\"no\"?>\n");
    out.push_str("<customField>\n");

    for v in vertices {
        out.push_str(&format!("    <vertex>{} {}</vertex>\n", v.x, v.y));
    }

    // Ring schliessen: letzter == erster
    if let Some(first) = vertices.first() {
        out.push_str(&format!("    <vertex>{} {}</vertex>\n", first.x, first.y));
    }

    out.push_str("</customField>\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_XML: &str = r#"<?xml version="1.0" encoding="utf-8" standalone="no"?>
<customField>
    <vertex>-431.267 -579.803</vertex>
    <vertex>-425.806 -576.663</vertex>
    <vertex>-420.000 -570.000</vertex>
    <vertex>-431.267 -579.803</vertex>
</customField>"#;

    #[test]
    fn test_parse_removes_ring_closer() {
        let verts = parse_curseplay(SAMPLE_XML).expect("Parsen sollte klappen");
        // 3 Vertices (letzter == erster wird entfernt)
        assert_eq!(verts.len(), 3);
        assert!((verts[0].x - (-431.267_f32)).abs() < 0.01);
    }

    #[test]
    fn test_roundtrip() {
        let original = vec![
            Vec2::new(1.0, 2.0),
            Vec2::new(3.0, 4.0),
            Vec2::new(5.0, 6.0),
        ];
        let xml = write_curseplay(&original);
        let parsed = parse_curseplay(&xml).expect("Roundtrip muss klappen");
        assert_eq!(parsed.len(), original.len());
        for (a, b) in original.iter().zip(parsed.iter()) {
            assert!(
                (a.x - b.x).abs() < 0.001,
                "x abweichend: {} vs {}",
                a.x,
                b.x
            );
            assert!(
                (a.y - b.y).abs() < 0.001,
                "y abweichend: {} vs {}",
                a.y,
                b.y
            );
        }
    }

    #[test]
    fn test_parse_error_on_empty() {
        let xml = "<customField></customField>";
        assert!(parse_curseplay(xml).is_err());
    }

    #[test]
    fn test_parse_error_on_invalid_coord() {
        let xml = "<customField><vertex>abc 1.0</vertex></customField>";
        assert!(parse_curseplay(xml).is_err());
    }

    #[test]
    fn test_write_includes_ring_closer() {
        let verts = vec![Vec2::new(1.0, 2.0), Vec2::new(3.0, 4.0)];
        let xml = write_curseplay(&verts);
        // Zwei Vertices + Ringschluss = 3 <vertex>-Tags
        assert_eq!(xml.matches("<vertex>").count(), 3);
    }
}
