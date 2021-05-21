use abc_parser::datatypes::{MusicSymbol, Note, Tune, TuneHeader};
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
                    let frequency = get_frequency(*note, *octave);
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

fn parse_fraction(s: &str) -> Result<f32, Report> {
    let (numerator, denominator) = s
        .split_once('/')
        .ok_or_else(|| eyre!("Invalid fraction {}", s))?;
    let numerator: f32 = numerator.parse()?;
    let denominator: f32 = denominator.parse()?;
    Ok(numerator / denominator)
}

fn get_frequency(note: Note, octave: i8) -> NonZeroU32 {
    let frequency = match note {
        Note::C => 261.63,
        Note::D => 293.66,
        Note::E => 329.63,
        Note::F => 349.23,
        Note::G => 392.00,
        Note::A => 440.00,
        Note::B => 493.88,
    };
    let frequency = frequency * 2.0f32.powi(octave as i32 - 1);
    NonZeroU32::new(frequency.round() as u32).unwrap()
}
