use super::*;

#[test]
fn base64_round_trips_every_length() {
    for n in 0..70u8 {
        let bytes: Vec<u8> = (0..n).map(|i| i.wrapping_mul(37) ^ n).collect();
        assert_eq!(unbase64(&og_probe::base64::encode(&bytes)).unwrap(), bytes);
    }
    assert!(unbase64("ab!c").is_none());
}

#[test]
fn a_save_decodes_to_its_snapshot_and_other_text_is_refused_with_the_fix() {
    let save = encode(b"OGWS world bytes", 42, 3, None);
    assert!(save.starts_with("og-save:3:"));
    let read = decode_save(&save).unwrap();
    assert_eq!(read.snapshot, b"OGWS world bytes");
    assert_eq!(
        (read.tick, read.version),
        (42, 3),
        "the game's save version"
    );
    assert!(
        upgraded(&save, 2).unwrap_err().contains("save version 3"),
        "a newer game's save is refused"
    );
    assert!(decode("{\"chimes\":1}").unwrap_err().contains("not a save"));
    assert!(
        decode("og-save:4:AAAA")
            .unwrap_err()
            .contains("save format 4 is incompatible with engine save format 3")
    );
    assert!(decode("og-save:3:@@@").unwrap_err().contains("damaged"));
    assert!(decode("og-save:3:AAAA").unwrap_err().contains("damaged"));
}
