"""Offline check of fetch_teams' lineup picking: python3 scripts/test_fetch_teams.py"""

import unittest

import fetch_teams as ft


def fixture(fid, ts, home, away):
    team = lambda i: {"id": i, "name": f"T{i}"}
    return {"fixture": {"id": fid, "timestamp": ts}, "teams": {"home": team(home), "away": team(away)}}


def lineup(tid, formation, n=11):
    p = {"player": {"name": f"P{tid}", "number": 7, "grid": "1:1"}}
    return {"team": {"id": tid}, "formation": formation, "startXI": [p] * n}


class LeagueTeams(unittest.TestCase):
    def test_newest_lineup_per_team_and_one_call_per_match(self):
        lineups = {
            1: [lineup(1, "4-4-2"), lineup(2, "4-3-3")],  # old
            2: [lineup(1, "3-5-2"), lineup(3, "4-2-3-1", n=0)],  # newest; team 3 has no XI
            3: [lineup(3, "5-3-2"), lineup(2, "4-1-4-1")],
        }
        calls = []
        ft.get = lambda key, path, fixture: calls.append(fixture) or lineups[fixture]
        got = ft.league_teams("k", [fixture(1, 10, 1, 2), fixture(2, 30, 1, 3), fixture(3, 20, 3, 2)])
        self.assertEqual([(t["name"], t["formation"]) for t in got],
                         [("T1", "3-5-2"), ("T2", "4-1-4-1"), ("T3", "5-3-2")])
        self.assertEqual(calls, [2, 3])


if __name__ == "__main__":
    unittest.main()
