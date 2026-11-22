//! Formations as grid cells on a vertical pitch.
//! Column 0..COLS is left→right, row 0..ROWS is the opponent's goal line → our goal line.

pub const COLS: u8 = 9;
pub const ROWS: u8 = 13;

pub struct Formation {
    pub name: &'static str,
    /// (col, row) per player, goalkeeper first.
    pub slots: [(u8, u8); 11],
}

const GK: (u8, u8) = (4, 12);
const BACK4: [(u8, u8); 4] = [(1, 10), (3, 10), (5, 10), (7, 10)];
const BACK3: [(u8, u8); 3] = [(2, 10), (4, 10), (6, 10)];

const fn f(name: &'static str, d: &[(u8, u8)], rest: &[(u8, u8)]) -> Formation {
    let mut slots = [GK; 11];
    let mut i = 0;
    while i < d.len() {
        slots[1 + i] = d[i];
        i += 1;
    }
    let mut j = 0;
    while j < rest.len() {
        slots[1 + d.len() + j] = rest[j];
        j += 1;
    }
    assert!(
        d.len() + rest.len() == 10,
        "a formation needs 10 outfield players"
    );
    Formation { name, slots }
}

pub const FORMATIONS: [Formation; 10] = [
    f(
        "4-3-3",
        &BACK4,
        &[(2, 7), (4, 8), (6, 7), (1, 4), (4, 3), (7, 4)],
    ),
    f(
        "4-2-3-1",
        &BACK4,
        &[(3, 8), (5, 8), (1, 5), (4, 5), (7, 5), (4, 3)],
    ),
    f(
        "4-4-2",
        &BACK4,
        &[(1, 7), (3, 7), (5, 7), (7, 7), (3, 4), (5, 4)],
    ),
    f(
        "3-5-2",
        &BACK3,
        &[(0, 7), (4, 8), (3, 6), (5, 6), (8, 7), (3, 3), (5, 3)],
    ),
    f(
        "4-1-4-1",
        &BACK4,
        &[(4, 8), (1, 6), (3, 6), (5, 6), (7, 6), (4, 3)],
    ),
    f(
        "4-4-1-1",
        &BACK4,
        &[(1, 7), (3, 7), (5, 7), (7, 7), (4, 5), (4, 3)],
    ),
    f(
        "3-4-3",
        &BACK3,
        &[(0, 7), (3, 7), (5, 7), (8, 7), (1, 4), (4, 3), (7, 4)],
    ),
    f(
        "5-3-2",
        &[(0, 9), (2, 10), (4, 10), (6, 10), (8, 9)],
        &[(2, 7), (4, 7), (6, 7), (3, 4), (5, 4)],
    ),
    f(
        "4-1-2-1-2",
        &BACK4,
        &[(4, 8), (2, 7), (6, 7), (4, 5), (3, 3), (5, 3)],
    ),
    f(
        "3-4-2-1",
        &BACK3,
        &[(0, 7), (3, 7), (5, 7), (8, 7), (2, 5), (6, 5), (4, 3)],
    ),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slots_are_on_grid_and_unique() {
        for f in &FORMATIONS {
            for (i, a) in f.slots.iter().enumerate() {
                assert!(a.0 < COLS && a.1 < ROWS, "{} slot {i} off grid", f.name);
                assert!(
                    !f.slots[i + 1..].contains(a),
                    "{} has duplicate slot {a:?}",
                    f.name
                );
            }
        }
    }
}
