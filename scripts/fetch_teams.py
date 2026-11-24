#!/usr/bin/env python3
"""Writes data/teams.json: each team's most recent starting XI and formation, from API-Football.

Env: API_FOOTBALL_KEY (required; without it this exits 0 and changes nothing),
API_FOOTBALL_SEASON (optional starting year, e.g. 2024; default: the current season,
stepping back while the plan refuses it, since the free plan only serves older seasons).

Cost: 1 request per league for its finished fixtures, then 1 per match lineup, and one
lineup call covers both teams, so the last round of matches covers most of a league.
About 35 requests in all; the free plan allows 100 a day.
Standard library only.
"""

import datetime
import json
import os
import sys
import time
import urllib.parse
import urllib.request

LEAGUES = {"Liga Portugal": 94, "Premier League": 39, "La Liga": 140}
OUT = os.path.join(os.path.dirname(__file__), "..", "data", "teams.json")
MAX_REQUESTS = 90  # stay under the free plan's 100/day
PAUSE = 6.5  # seconds between calls; the free plan allows 10 per minute

used = 0


class ApiError(Exception):
    pass


def get(key, path, **params):
    global used
    if used >= MAX_REQUESTS:
        raise ApiError(f"request budget of {MAX_REQUESTS} used up")
    if used:
        time.sleep(PAUSE)
    used += 1
    url = f"https://v3.football.api-sports.io/{path}?{urllib.parse.urlencode(params)}"
    req = urllib.request.Request(url, headers={"x-apisports-key": key})
    with urllib.request.urlopen(req, timeout=30) as r:
        body = json.load(r)
    if body.get("errors"):  # [] or {} when fine
        raise ApiError(f"{path} {params}: {body['errors']}")
    return body["response"]


def finished(key, league, season):
    return get(key, "fixtures", league=league, season=season, status="FT")


def find_season(key, league):
    """Current season, or the newest one the plan serves that has finished matches."""
    now = datetime.date.today()
    season = now.year if now.month >= 7 else now.year - 1
    for s in range(season, season - 4, -1):
        try:
            fixtures = finished(key, league, s)
        except ApiError as e:
            if "plan" not in str(e).lower() and "season" not in str(e).lower():
                raise
            print(f"season {s}: {e}")
            continue
        if fixtures:
            return s, fixtures
        print(f"season {s}: no finished matches yet")
    raise ApiError("no usable season found; set API_FOOTBALL_SEASON")


def xi(lineup):
    return [
        {
            "name": p["player"]["name"] or "",
            "number": max(0, min(99, p["player"]["number"] or 0)),
            "grid": p["player"]["grid"] or "",
        }
        for p in lineup.get("startXI") or []
    ]


def league_teams(key, fixtures):
    """Latest usable lineup per team, newest matches first."""
    fixtures.sort(key=lambda f: f["fixture"]["timestamp"], reverse=True)
    names = {}
    for f in fixtures:
        for side in ("home", "away"):
            names[f["teams"][side]["id"]] = f["teams"][side]["name"]
    found = {}
    for f in fixtures:
        if len(found) == len(names):
            break
        ids = [f["teams"][s]["id"] for s in ("home", "away")]
        if all(i in found for i in ids):
            continue
        for lu in get(key, "fixtures/lineups", fixture=f["fixture"]["id"]):
            tid = lu["team"]["id"]
            players = xi(lu)
            if tid in names and tid not in found and lu.get("formation") and len(players) == 11:
                found[tid] = {"name": names[tid], "formation": lu["formation"], "xi": players}
    missing = sorted(names[i] for i in names if i not in found)
    if missing:
        print(f"  no lineup for: {', '.join(missing)}")
    return sorted(found.values(), key=lambda t: t["name"])


def main():
    key = os.environ.get("API_FOOTBALL_KEY", "").strip()
    if not key:
        print("::notice::API_FOOTBALL_KEY is not set; teams.json left unchanged.")
        return 0
    fixed = os.environ.get("API_FOOTBALL_SEASON", "").strip()
    season = int(fixed) if fixed else None
    leagues = []
    try:
        for name, lid in LEAGUES.items():
            if season is None:
                season, fixtures = find_season(key, lid)
            else:
                fixtures = finished(key, lid, season)
            print(f"{name} {season}: {len(fixtures)} finished matches")
            leagues.append({"name": name, "teams": league_teams(key, fixtures)})
    except (ApiError, OSError) as e:
        print(f"::error::{e} (teams.json left unchanged)")
        return 1
    finally:
        print(f"{used} requests used")
    # Only written once everything succeeded, so a failed run never leaves half a file.
    with open(OUT, "w", encoding="utf-8") as f:
        json.dump({"season": season, "leagues": leagues}, f, ensure_ascii=False, indent=1)
        f.write("\n")
    return 0


if __name__ == "__main__":
    sys.exit(main())
