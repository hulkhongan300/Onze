//! Real starting XIs, fetched by `scripts/fetch_teams.py` into `data/teams.json` and compiled in.

use crate::formations::FORMATIONS;
use serde::Deserialize;
use std::sync::LazyLock;

#[derive(Default, Deserialize)]
pub struct Data {
    /// Season the lineups come from, by starting year (2024 = 2024/25).
    pub season: Option<u16>,
    pub leagues: Vec<League>,
}

#[derive(Deserialize)]
pub struct League {
    pub name: String,
    pub teams: Vec<Team>,
}

#[derive(Deserialize)]
pub struct Team {
    pub name: String,
    /// As API-Football writes it, e.g. "4-2-3-1".
    pub formation: String,
    pub xi: Vec<Pick>,
}

#[derive(Deserialize)]
pub struct Pick {
    pub name: String,
    pub number: u8,
    /// API-Football "row:col": row 1 is the keeper, rows go towards attack,
    /// col 1 is the team's left.
    pub grid: String,
}

pub static DATA: LazyLock<Data> = LazyLock::new(|| parse(include_str!("../data/teams.json")));

/// Bad JSON gives no teams; teams without exactly 11 starters and empty leagues are dropped.
fn parse(json: &str) -> Data {
    let mut d: Data = serde_json::from_str(json).unwrap_or_default();
    for l in &mut d.leagues {
        l.teams.retain(|t| t.xi.len() == 11);
    }
    d.leagues.retain(|l| !l.teams.is_empty());
    d
}

fn grid(p: &Pick) -> Option<(u8, u8)> {
    let (r, c) = p.grid.split_once(':')?;
    Some((r.trim().parse().ok()?, c.trim().parse().ok()?))
}

/// Where each starter goes: the formation to use (`None` if the team's isn't one of ours,
/// then `current` is kept) and, for each of our 11 slots, the index into `team.xi`.
/// `team.xi` must have 11 entries.
pub fn assign(team: &Team, current: usize) -> (Option<usize>, [usize; 11]) {
    let known = FORMATIONS.iter().position(|f| f.name == team.formation);
    let order = known
        .and_then(|f| by_lines(team, f))
        .unwrap_or_else(|| back_to_front(team, known.unwrap_or(current)));
    (known, order)
}

/// Exact mapping: our slots list the formation's lines in order (keeper first), so API row r
/// is our line r-1, and col c is the c-th slot from the left in that line.
fn by_lines(team: &Team, f: usize) -> Option<[usize; 11]> {
    let slots = &FORMATIONS[f].slots;
    let mut lines = vec![vec![0]];
    let mut start = 1;
    for size in FORMATIONS[f].name.split('-') {
        let size: usize = size.parse().ok()?;
        let mut line: Vec<usize> = (start..start + size).collect();
        line.sort_by_key(|&i| slots[i].0);
        lines.push(line);
        start += size;
    }
    let mut out = [usize::MAX; 11];
    for (j, p) in team.xi.iter().enumerate() {
        let (r, c) = grid(p)?;
        let slot = *lines
            .get(r.checked_sub(1)? as usize)?
            .get(c.checked_sub(1)? as usize)?;
        if out[slot] != usize::MAX {
            return None;
        }
        out[slot] = j;
    }
    Some(out)
}

/// Fallback: starters by (row, col), our slots from our goal outwards and left to right.
fn back_to_front(team: &Team, f: usize) -> [usize; 11] {
    let slots = &FORMATIONS[f].slots;
    let mut picks: Vec<usize> = (0..11).collect();
    // Unreadable grids sort last, in list order.
    picks.sort_by_key(|&j| grid(&team.xi[j]).unwrap_or((u8::MAX, u8::MAX)));
    let mut ours: Vec<usize> = (0..11).collect();
    ours.sort_by_key(|&i| (std::cmp::Reverse(slots[i].1), slots[i].0));
    let mut out = [0; 11];
    for (slot, j) in ours.into_iter().zip(picks) {
        out[slot] = j;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn team(formation: &str, grids: &[&str; 11]) -> Team {
        Team {
            name: "T".into(),
            formation: formation.into(),
            xi: grids
                .iter()
                .map(|g| Pick {
                    name: (*g).into(),
                    number: 0,
                    grid: (*g).into(),
                })
                .collect(),
        }
    }

    /// Grid of the player placed in each of our slots.
    fn placed(t: &Team, order: [usize; 11]) -> Vec<&str> {
        order.iter().map(|&j| t.xi[j].grid.as_str()).collect()
    }

    // Italy's 4-3-3 lineup as API-Football lists it: right-back first in each line.
    const ITALY: [&str; 11] = [
        "1:1", "2:4", "2:3", "2:2", "2:1", "3:3", "3:2", "3:1", "4:3", "4:2", "4:1",
    ];

    #[test]
    fn four_three_three_lands_on_matching_slots() {
        let t = team("4-3-3", &ITALY);
        let (f, order) = assign(&t, 5);
        assert_eq!(f, Some(0));
        // Our 4-3-3 slots: GK, LB, LCB, RCB, RB, LCM (2,7), CDM (4,8), RCM (6,7), LW, ST, RW.
        assert_eq!(
            placed(&t, order),
            [
                "1:1", "2:1", "2:2", "2:3", "2:4", "3:1", "3:2", "3:3", "4:1", "4:2", "4:3"
            ]
        );
    }

    #[test]
    fn lines_are_sorted_left_to_right_by_our_columns() {
        // Our 3-5-2 midfield is listed as (0,7), (4,8), (3,6), (5,6), (8,7).
        let t = team(
            "3-5-2",
            &[
                "1:1", "2:1", "2:2", "2:3", "3:1", "3:2", "3:3", "3:4", "3:5", "4:1", "4:2",
            ],
        );
        let (f, order) = assign(&t, 0);
        assert_eq!(FORMATIONS[f.unwrap()].name, "3-5-2");
        assert_eq!(placed(&t, order)[4..9], ["3:1", "3:3", "3:2", "3:4", "3:5"]);
    }

    #[test]
    fn unknown_formation_keeps_current_and_fills_back_to_front() {
        let t = team(
            "3-1-4-2",
            &[
                "4:2", "1:1", "2:1", "2:2", "2:3", "3:1", "4:1", "4:3", "4:4", "5:1", "5:2",
            ],
        );
        let (f, order) = assign(&t, 2);
        assert_eq!(f, None);
        let got = placed(&t, order);
        // 4-4-2 keeper and back four.
        assert_eq!(got[0], "1:1");
        assert_eq!(got[1..5], ["2:1", "2:2", "2:3", "3:1"]);
        let mut seen = got.clone();
        seen.sort();
        seen.dedup();
        assert_eq!(seen.len(), 11, "every starter placed once");
    }

    #[test]
    fn bad_grid_in_known_formation_still_places_everyone() {
        let mut g = ITALY;
        g[3] = "9:9";
        let t = team("4-3-3", &g);
        let (f, order) = assign(&t, 1);
        assert_eq!(f, Some(0));
        let mut seen = order.to_vec();
        seen.sort();
        assert_eq!(seen, (0..11).collect::<Vec<_>>());
    }

    #[test]
    fn empty_or_broken_data_gives_no_teams() {
        assert!(parse("").leagues.is_empty());
        assert!(parse("{\"leagues\": 3}").leagues.is_empty());
        let d = parse(include_str!("../data/teams.json"));
        assert!(
            d.leagues
                .iter()
                .all(|l| l.teams.iter().all(|t| t.xi.len() == 11))
        );
        let short =
            r#"{"leagues":[{"name":"L","teams":[{"name":"T","formation":"4-4-2","xi":[]}]}]}"#;
        assert!(parse(short).leagues.is_empty());
    }
}
