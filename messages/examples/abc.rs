use abc_parser::datatypes::{Accidental, MusicSymbol, Note, Tune, TuneHeader};
use eyre::{eyre, Report};
use log::error;
use messages::client::Hoverkite;
use std::env;
use std::num::NonZeroU32;
use std::process::exit;
use std::thread;
use std::time::Duration;

const BAUD_RATE: u32 = 115_200;
const SLEEP_DURATION: Duration = Duration::from_millis(2);

fn main() -> Result<(), Report> {
    stable_eyre::install()?;
    pretty_env_logger::init();
    color_backtrace::install();

    let mut args = env::args();
    let binary_name = args
        .next()
        .ok_or_else(|| eyre::eyre!("Binary name missing"))?;
    if args.len() != 1 {
        eprintln!("Usage:");
        eprintln!("  {} <serial port>", binary_name);
        exit(1);
    }
    let port_name = args.next().unwrap();

    let port = serialport::new(&port_name, BAUD_RATE)
        .open()
        .map_err(|e| error!("Failed to open serial port {}: {}", port_name, e))
        .ok();

    let mut hoverkite = Hoverkite::new(port, None);

    let tune = abc_parser::abc::tune(
        "X:1
T: The Origin Of The World
R: mazurka
M: 3/4
L: 1/8
K: Gmin
|: de dc AB | G2-GGAB | ce ee dc | d2-dd dc |
de dc AB | G2-GG AB | EG BE GB | A2 AA BA |
GE CE GE | F2 FD B,D | GE B,E GE | F2 F2 GA |
B2 Bc-cd | d2-dc Bc | cc cB GF | G2 G4 :|
",
    )?;
    let notes = abc_to_notes(tune)?;
    hoverkite.play_notes(&notes)?;

    loop {
        for response in hoverkite.poll()? {
            println!("{:?}", response);
        }

        thread::sleep(SLEEP_DURATION);
    }
}

/// Tempo in BPM.
const TEMPO: f32 = 200.0;
/// The duration in milliseconds of a whole note.
const WHOLE_NOTE_DURATION: f32 = 60.0 * 1000.0 * 4.0 / TEMPO;

fn abc_to_notes(tune: Tune) -> Result<Vec<messages::Note>, Report> {
    let mut notes = vec![];
    println!("Header: {:?}", tune.header);
    let base_duration = get_base_duration(&tune.header)?;
    println!(
        "Base duration: {} ms (crotchet = {} ms)",
        base_duration,
        WHOLE_NOTE_DURATION / 4.0
    );
    let key_signature = get_key_signature(&tune.header)?;
    println!("Key signature: {}", key_signature);
    let body = tune.body.ok_or_else(|| eyre!("Tune has no body"))?;
    for line in &body.music {
        for symbol in &line.symbols {
            match symbol {
                MusicSymbol::Note {
                    decorations,
                    accidental,
                    note,
                    octave,
                    length,
                    tie,
                } => {
                    let frequency = get_frequency(*note, *octave, *accidental, key_signature);
                    println!("note: {:?}{} ({}) {}", note, octave, frequency, length);
                    notes.push(messages::Note {
                        frequency: Some(frequency),
                        duration_ms: (base_duration * length) as u32,
                    });
                }
                _ => println!("symbol: {:?}", symbol),
            }
        }
    }
    Ok(notes)
}

/// Figure out the duration in milliseconds of a length-1 note.
fn get_base_duration(header: &TuneHeader) -> Result<f32, Report> {
    let length_field = header
        .info
        .iter()
        .find(|info| info.0 == 'L')
        .ok_or_else(|| eyre!("Header field L missing"))?;
    let length = parse_fraction(&length_field.1)?;
    Ok(length * WHOLE_NOTE_DURATION)
}

fn get_key_signature(header: &TuneHeader) -> Result<i8, Report> {
    let key_signature_field = header
        .info
        .iter()
        .find(|info| info.0 == 'K')
        .ok_or_else(|| eyre!("Header field K missing"))?;
    key_signature(&key_signature_field.1)
}

fn parse_fraction(s: &str) -> Result<f32, Report> {
    let (numerator, denominator) = s
        .split_once('/')
        .ok_or_else(|| eyre!("Invalid fraction {}", s))?;
    let numerator: f32 = numerator.parse()?;
    let denominator: f32 = denominator.parse()?;
    Ok(numerator / denominator)
}

// Middle C is 0
fn get_semitone(note: Note, accidental: Option<Accidental>, key_signature: i8) -> i32 {
    let accidental = accidental.unwrap_or_else(|| match note {
        Note::B if key_signature >= 7 => Accidental::Sharp,
        Note::E if key_signature >= 6 => Accidental::Sharp,
        Note::A if key_signature >= 5 => Accidental::Sharp,
        Note::D if key_signature >= 4 => Accidental::Sharp,
        Note::G if key_signature >= 3 => Accidental::Sharp,
        Note::C if key_signature >= 2 => Accidental::Sharp,
        Note::F if key_signature >= 1 => Accidental::Sharp,
        Note::B if key_signature <= -1 => Accidental::Flat,
        Note::E if key_signature <= -2 => Accidental::Flat,
        Note::A if key_signature <= -3 => Accidental::Flat,
        Note::D if key_signature <= -4 => Accidental::Flat,
        Note::G if key_signature <= -5 => Accidental::Flat,
        Note::C if key_signature <= -6 => Accidental::Flat,
        Note::F if key_signature <= -7 => Accidental::Flat,
        _ => Accidental::Natural,
    });
    let semitone = match note {
        Note::C => 0,
        Note::D => 2,
        Note::E => 4,
        Note::F => 5,
        Note::G => 7,
        Note::A => 9,
        Note::B => 11,
    };
    match accidental {
        Accidental::DoubleFlat => semitone - 2,
        Accidental::Flat => semitone - 1,
        Accidental::Natural => semitone,
        Accidental::Sharp => semitone + 1,
        Accidental::DoubleSharp => semitone + 2,
    }
}

fn get_frequency(
    note: Note,
    octave: i8,
    accidental: Option<Accidental>,
    key_signature: i8,
) -> NonZeroU32 {
    let semitone = get_semitone(note, accidental, key_signature);
    // The A above middle C (tone 9) is 440 Hz.
    let frequency = 440.0 * 2.0f32.powf(octave as f32 - 1.0 + (semitone - 9) as f32 / 12.0);
    NonZeroU32::new(frequency.round() as u32).unwrap()
}

/// Returns a positive number of sharps, or a negative number of flats (or 0 for neither).
fn key_signature(signature: &str) -> Result<i8, Report> {
    match signature {
        "C#" | "A#m" | "G#Mix" | "D#Dor" | "E#Phr" | "F#Lyd" | "B#Loc" => Ok(7),
        "F#" | "D#m" | "C#Mix" | "G#Dor" | "A#Phr" | "BLyd" | "E#Loc" => Ok(6),
        "B" | "G#m" | "F#Mix" | "C#Dor" | "D#Phr" | "ELyd" | "A#Loc" => Ok(5),
        "E" | "C#m" | "BMix" | "F#Dor" | "G#Phr" | "ALyd" | "D#Loc" => Ok(4),
        "A" | "F#m" | "EMix" | "BDor" | "C#Phr" | "DLyd" | "G#Loc" => Ok(3),
        "D" | "Bm" | "AMix" | "EDor" | "F#Phr" | "GLyd" | "C#Loc" => Ok(2),
        "G" | "Em" | "DMix" | "ADor" | "BPhr" | "CLyd" | "F#Loc" => Ok(1),
        "C" | "Am" | "GMix" | "DDor" | "EPhr" | "FLyd" | "BLoc" => Ok(0),
        "F" | "Dm" | "CMix" | "GDor" | "APhr" | "BbLyd" | "ELoc" => Ok(-1),
        "Bb" | "Gm" | "FMix" | "CDor" | "DPhr" | "EbLyd" | "ALoc" => Ok(-2),
        "Eb" | "Cm" | "BbMix" | "FDor" | "GPhr" | "AbLyd" | "DLoc" => Ok(-3),
        "Ab" | "Fm" | "EbMix" | "BbDor" | "CPhr" | "DbLyd" | "GLoc" => Ok(-4),
        "Db" | "Bbm" | "AbMix" | "EbDor" | "FPhr" | "GbLyd" | "CLoc" => Ok(-5),
        "Gb" | "Ebm" | "DbMix" | "AbDor" | "BbPhr" | "CbLyd" | "FLoc" => Ok(-6),
        "Cb" | "Abm" | "GbMix" | "DbDor" | "EbPhr" | "FbLyd" | "BbLoc" => Ok(-7),
        _ => Err(eyre!("Invalid key signature {}", signature)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn c_major_note_frequencies() {
        assert_eq!(get_frequency(Note::C, 1, None, 0).get(), 262);
        assert_eq!(get_frequency(Note::A, 1, None, 0).get(), 440);
        assert_eq!(get_frequency(Note::B, 1, None, 0).get(), 494);
    }

    #[test]
    fn f_key_frequencies() {
        // Check the frequency in C major, with various accidentals.
        assert_eq!(get_frequency(Note::F, 1, None, 0).get(), 349);
        assert_eq!(
            get_frequency(Note::F, 1, Some(Accidental::Natural), 0).get(),
            349
        );
        assert_eq!(
            get_frequency(Note::F, 1, Some(Accidental::Sharp), 0).get(),
            370
        );
        assert_eq!(
            get_frequency(Note::F, 1, Some(Accidental::Flat), 0).get(),
            330
        );

        // Check the frequencies in various keys with no accidentals.
        assert_eq!(get_frequency(Note::F, 1, None, 1).get(), 370);
        assert_eq!(get_frequency(Note::F, 1, None, 7).get(), 370);
        assert_eq!(get_frequency(Note::F, 1, None, -6).get(), 349);
        assert_eq!(get_frequency(Note::F, 1, None, -7).get(), 330);

        // A natural accidental should mean that the key is ignored.
        assert_eq!(
            get_frequency(Note::F, 1, Some(Accidental::Natural), 7).get(),
            349
        );
        assert_eq!(
            get_frequency(Note::F, 1, Some(Accidental::Natural), -7).get(),
            349
        );
    }

    #[test]
    fn get_key_signature_success() {
        let tune = abc_parser::abc::tune("X:1\nT:blah\nK:Gm\n").unwrap();
        let key_signature = get_key_signature(&tune.header).unwrap();
        assert_eq!(key_signature, -2);
    }
}
