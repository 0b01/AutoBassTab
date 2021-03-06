//! Algorithm for arranging notes on bass
//! takes in a csv of notes with header
//! and outputs csv of fretboard positions with header

extern crate csv;

use std::collections::HashMap;
const MIDI_A4: f32 = 69.;
const FREQ_A4: f32 = 440.;

/// string no, fret no
type Pos = (u8, u8);

#[repr(usize)]
enum Chroma {
    C = 0,
    Db,
    D,
    Eb,
    E,
    F,
    Gb,
    G,
    Ab,
    A,
    Bb,
    B,
}

#[derive(Clone, Hash, Eq, PartialEq, Copy, Debug)]
struct Note {
    idx: isize,
    octave: isize,
}

/// biomechanical cost of moving finger between consecutive notes
/// also penalizes open strings(which you should never play)
/// also penalizes high frets
fn cost(this: &Pos, other: &Pos) -> f32 {
    let (a_s, a_f) = *this;
    let (b_s, b_f) = *other;
    let a_s = a_s as f32;
    let b_s = b_s as f32;
    let a_f = a_f as f32;
    let b_f = b_f as f32;
    let mut c =
        (a_f - b_f).abs() + // fret distance
        (a_s - b_s).abs() * 0.3 + // string distance
        (a_f + b_f) * 0.3 + // fret position higher is bad
        (a_s + b_s) * 0.5 // penalize high strings
    ;
    if a_f == 0. || b_f == 0. {
        c += 8.;
    }
    c
}


impl Note {
    const PITCHES: [&'static str; 12] = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
    fn new(idx: isize, octave: isize) -> Self {
        Self {
            idx, octave
        }
    }

    fn add(&self, semitones: isize) -> Self {
        let num_chroma = 12;
        let mut note = self.clone();
        note.idx = (note.idx + semitones) % num_chroma;
        while note.idx < 0 { note.idx += 12; }
        let oct_diff = semitones / 12;
        note.octave = self.octave + oct_diff;
        if semitones > 0 {
            if note.idx >= 0 && note.idx < self.idx {
                note.octave += 1;
            }
        } else if note.idx > self.idx && note.idx < num_chroma {
            note.octave -= 1;
        }
        note
    }

    fn to_string(&self) -> String {
        let mut ret = String::new();
        ret += Note::PITCHES[self.idx as usize];
        ret += &self.octave.to_string();
        return ret;
    }
}

fn freq_to_midi(freq: f32) -> f32 {
    12. * (freq.log2() - FREQ_A4.log2()) + MIDI_A4
}

fn freq_to_note(freq: f32) -> Note {
    let midi_number = freq_to_midi(freq);
    let num = midi_number - (MIDI_A4 - 4. * 12. - 9.);
    let note = (num + 0.5) % 12. - 0.5;
    let rnote = note.round() as isize;
    let octave = ((num - note) / 12.).round() as isize;
    Note::new(rnote, octave)
}

macro_rules! note {
    ($t:tt, $num: expr) => {
        Note::new(Chroma::$t as isize, $num)
    }
}

struct Arranger {
    strings: [Note; 4],
    neck: HashMap<Pos, Note>,
    pos: HashMap<Note, Vec<Pos>>,
    cache: HashMap<Key, (f32, Vec<(Pos, Pos)>)>
}

type Key = (usize, usize, Option<Pos>, Option<Pos>);

impl Arranger {
    fn init() -> Self {
        let strings = [note!(E,1), note!(A,1), note!(D,2), note!(G,2)];
        let neck = {
            let mut temp = HashMap::new();
            for (i, note) in strings.iter().enumerate() {
                for j in 0..21 {
                    temp.insert((i as u8, j as u8), note.add(j));
                }
            }
            temp
        };

        let pos = {
            let mut temp = HashMap::new();
            for (k, v) in &neck {
                let arr = temp.entry(*v).or_insert(Vec::new());
                (*arr).push(*k);
            }
            temp
        };
        let cache = HashMap::new();
        Self {
            cache,
            strings,
            neck,
            pos,
        }
    }

    fn reset(&mut self) {
        self.cache.clear();
    }

    /// arrange 2 notes
    fn arrange2(&self, n1: &Note, n2: &Note, left: Option<Pos>, right: Option<Pos>) -> (f32, (Pos, Pos)) {
        let mut curr_min = 1000000000.;
        let mut temp = None;
        let left = left.map(|x|vec![x]).unwrap_or_else(|| self.pos.get(&n1).cloned().unwrap_or_else(|| vec![(0,0)] ));
        let right = right.map(|x|vec![x]).unwrap_or_else(|| self.pos.get(&n2).cloned().unwrap_or_else(|| vec![(0,0)] ));
        for pa in &left {
            for pb in &right {
                let c = cost(pa, pb);
                if c < curr_min {
                    curr_min = c;
                    temp = Some((*pa, *pb));
                }
            }
        }
        return (curr_min, temp.unwrap())
    }

    fn arrange_aux(&mut self, notes: &[Note], i: usize, j: usize, left: Option<Pos>, right: Option<Pos>)
        -> (f32, Vec<(Pos, Pos)>)
    {
        let key = (i,j,left,right);
        if self.cache.contains_key(&key) {
            return (self.cache.get(&key).unwrap()).clone();
        }
        let mut curr_min = 100000000000.;
        let mut arr = None;
        if i >= j {
            return (0., vec![]);
        }
        for k in i..j {
            let (c_curr, (l, r)) = if left.is_some() && k == i {
                self.arrange2(&notes[k], &notes[k+1], left, None)
            } else if right.is_some() && k == j {
                self.arrange2(&notes[k], &notes[k+1], None, right)
            } else {
                self.arrange2(&notes[k], &notes[k+1], None, None)
            };
            let (c1, s1) = self.arrange_aux(notes, i, k, None, Some(l));
            let (c2, s2) = self.arrange_aux(notes, k+1, j, Some(r), None);
            let c = c_curr + c1 + c2;
            if c < curr_min {
                curr_min = c;
                let mut temp = vec![];
                temp.extend(&s1);
                temp.push((l,r));
                temp.extend(&s2);
                arr = Some(temp);
            }
        }
        let ret = (curr_min, arr.unwrap());
        self.cache.insert(key, ret.clone());
        ret
    }

    fn arrange(&mut self, notes: &[Note]) -> (f32, Vec<(Pos, Pos)>) {
        self.arrange_aux(&notes, 0, notes.len()-1, None, None)
    }
}
use std::io;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}


#[wasm_bindgen]
pub fn frets(semitone_delta: isize, out: &mut[u8], input: &[f32]) {
    let mut freqs = vec![];
    let mut starts = vec![];
    let mut durs = vec![];
    let mut it = input.iter();
    while let Some(i) = it.next() {
        freqs.push(*i);
        starts.push(it.next().unwrap());
        durs.push(it.next().unwrap());
    }

    let notes: Vec<_> = freqs.iter().copied()
        .map(freq_to_note)
        .map(|n| n.add(semitone_delta))
        .collect();

    for note in &notes {
        log(&note.to_string());
    }

    // TODO: time based grouping

    let mut arranger = Arranger::init();
    // println!("Arranging {} notes.", notes.len());
    let mut result = vec![];
    for chunk in notes.as_slice().chunks(200) {
        let (_cost, arrangement) = arranger.arrange(chunk);
        for (a, _b) in &arrangement {
            result.push(*a);
        }
        result.push(arrangement[arrangement.len()-1].1);
        arranger.reset();
    }

    // for (p, n) in result.iter().zip(&notes) { assert_eq!(arranger.neck[p], *n); }
    // assert_eq!(result.iter().map(|x|arranger.neck[x]).collect::<Vec<_>>(), notes);
    // assert_eq!(result.len(), notes.len());

    // println!("string,fret");
    let mut i = 0;
    for (string, fret) in result.iter() {
        out[i] = *string;
        i += 1;
        out[i] = *fret;
        i += 1;
        // println!("{},{}", string, fret);
    }
}

fn main() {

    let semitones_diff: isize = std::env::args().collect::<Vec<_>>()[1].parse().unwrap();

    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(io::stdin());
    let mut freqs = vec![];
    let mut starts = vec![];
    let mut durs = vec![];
    for result in rdr.records() {
        let record = result.unwrap();
        let freq: f32 = record.get(0).unwrap().parse().unwrap();
        let start: f32 = record.get(1).unwrap().parse().unwrap();
        let dur: f32 = record.get(2).unwrap().parse().unwrap();
        freqs.push(freq);
        starts.push(start);
        durs.push(dur);
    }

    let notes: Vec<_> = freqs.iter().copied()
        .map(freq_to_note)
        .map(|n| n.add(semitones_diff))
        .collect();

    // TODO: time based grouping

    let mut arranger = Arranger::init();
    // println!("Arranging {} notes.", notes.len());
    let mut result = vec![];
    for chunk in notes.as_slice().chunks(200) {
        let (_cost, arrangement) = arranger.arrange(chunk);
        for (a, _b) in &arrangement {
            result.push(*a);
        }
        result.push(arrangement[arrangement.len()-1].1);
        arranger.reset();
    }

    // for (p, n) in result.iter().zip(&notes) { assert_eq!(arranger.neck[p], *n); }
    // assert_eq!(result.iter().map(|x|arranger.neck[x]).collect::<Vec<_>>(), notes);
    // assert_eq!(result.len(), notes.len());

    println!("string,fret");
    for (string, fret) in result {
        println!("{},{}", string, fret);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_sub() {
        assert_eq!(note!(C,2).add(-1), note!(B,1));
        assert_eq!(note!(C,2).add(1), note!(Db,2));
        assert_eq!(note!(C,2).add(2), note!(D,2));
    }
}