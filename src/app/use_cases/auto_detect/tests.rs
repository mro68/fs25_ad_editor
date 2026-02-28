use super::*;
use std::fs;

#[test]
fn test_expand_umlaut_variants_with_umlauts() {
    let variants = expand_umlaut_variants("Höflingen");
    assert!(variants.contains(&"höflingen".to_string()));
    assert!(variants.contains(&"hoeflingen".to_string()));
}

#[test]
fn test_expand_umlaut_variants_with_ascii() {
    let variants = expand_umlaut_variants("Hoeflingen");
    assert!(variants.contains(&"hoeflingen".to_string()));
    assert!(variants.contains(&"höflingen".to_string()));
}

#[test]
fn test_expand_umlaut_variants_no_umlauts() {
    let variants = expand_umlaut_variants("Farm");
    assert_eq!(variants.len(), 1);
    assert!(variants.contains(&"farm".to_string()));
}

#[test]
fn test_name_to_pattern_spaces() {
    let pattern = name_to_pattern("big farm");
    assert_eq!(pattern, "big.*farm");
}

#[test]
fn test_name_to_pattern_underscores() {
    let pattern = name_to_pattern("big_farm");
    assert_eq!(pattern, "big.*farm");
}

#[test]
fn test_truncate_to_two_words() {
    assert_eq!(
        truncate_to_two_words("Sickinger_Hoehe_Rheinland_Pfalz"),
        "Sickinger_Hoehe"
    );
    assert_eq!(truncate_to_two_words("Big Farm West"), "Big_Farm");
    assert_eq!(truncate_to_two_words("SingleWord"), "SingleWord");
    assert_eq!(truncate_to_two_words("Two_Words"), "Two_Words");
}

#[test]
fn test_find_matching_zips() {
    let tmp = std::env::temp_dir().join("test_auto_detect_zips");
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).unwrap();

    // Testdateien anlegen
    fs::write(tmp.join("FS25_Hoeflingen.zip"), b"").unwrap();
    fs::write(tmp.join("FS25_Höflingen_V2.zip"), b"").unwrap();
    fs::write(tmp.join("FS25_Big_Farm.zip"), b"").unwrap();
    fs::write(tmp.join("FS25_Unrelated.zip"), b"").unwrap();
    fs::write(tmp.join("readme.txt"), b"").unwrap();
    fs::write(tmp.join("FS25_Sickinger_Hoehe_v3.zip"), b"").unwrap();

    // Test: "Höflingen" soll beide Höflingen-ZIPs finden
    let results = find_matching_zips(&tmp, "Höflingen");
    assert!(
        results
            .iter()
            .any(|p| p.file_name().unwrap().to_str().unwrap() == "FS25_Hoeflingen.zip"),
        "Soll FS25_Hoeflingen.zip finden, got: {:?}",
        results
    );
    assert!(
        results
            .iter()
            .any(|p| p.file_name().unwrap().to_str().unwrap() == "FS25_Höflingen_V2.zip"),
        "Soll FS25_Höflingen_V2.zip finden, got: {:?}",
        results
    );
    assert!(
        !results
            .iter()
            .any(|p| p.file_name().unwrap().to_str().unwrap() == "FS25_Unrelated.zip"),
        "Soll FS25_Unrelated.zip NICHT finden"
    );

    // Test: "Big Farm" soll FS25_Big_Farm.zip finden (Space → Underscore)
    let results2 = find_matching_zips(&tmp, "Big Farm");
    assert!(
        results2
            .iter()
            .any(|p| p.file_name().unwrap().to_str().unwrap() == "FS25_Big_Farm.zip"),
        "Soll FS25_Big_Farm.zip finden, got: {:?}",
        results2
    );

    // Test: Langer Name → nur erste 2 Wörter für Suche
    let results3 = find_matching_zips(&tmp, "Sickinger_Hoehe_Rheinland_Pfalz");
    assert!(
        results3
            .iter()
            .any(|p| p.file_name().unwrap().to_str().unwrap() == "FS25_Sickinger_Hoehe_v3.zip"),
        "Soll FS25_Sickinger_Hoehe_v3.zip finden, got: {:?}",
        results3
    );

    let _ = fs::remove_dir_all(&tmp);
}

#[test]
fn test_find_heightmap_next_to() {
    let tmp = std::env::temp_dir().join("test_auto_detect_hm");
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).unwrap();

    let xml_path = tmp.join("AutoDrive_config.xml");
    fs::write(&xml_path, b"<xml/>").unwrap();

    // Kein Heightmap → None
    assert!(find_heightmap_next_to(&xml_path).is_none());

    // Heightmap erstellen → Some
    let hm_path = tmp.join("terrain.heightmap.png");
    fs::write(&hm_path, b"PNG").unwrap();
    assert_eq!(find_heightmap_next_to(&xml_path), Some(hm_path));

    let _ = fs::remove_dir_all(&tmp);
}

#[test]
fn test_detect_post_load_integration() {
    let tmp = std::env::temp_dir().join("test_auto_detect_full");
    let _ = fs::remove_dir_all(&tmp);
    let savegame = tmp.join("savegame1");
    let mods = tmp.join("mods");
    fs::create_dir_all(&savegame).unwrap();
    fs::create_dir_all(&mods).unwrap();

    let xml_path = savegame.join("AutoDrive_config.xml");
    fs::write(&xml_path, b"<xml/>").unwrap();
    fs::write(savegame.join("terrain.heightmap.png"), b"PNG").unwrap();
    fs::write(mods.join("FS25_TestMap.zip"), b"").unwrap();

    let result = detect_post_load(&xml_path, Some("TestMap"));
    assert!(result.heightmap_path.is_some());
    assert_eq!(result.matching_zips.len(), 1);

    let _ = fs::remove_dir_all(&tmp);
}
