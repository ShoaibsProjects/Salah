#!/usr/bin/env python3
"""Optional online verification of USNO reference provenance; stdlib only.

This does not update the manifest or affect runtime prayer calculation.
The API can change, so a hash mismatch calls for source review even when
the selected solar event still matches.
"""

import argparse
import csv
import hashlib
import sys
from datetime import datetime, timedelta
from pathlib import Path
from urllib.parse import parse_qs, urlparse
from urllib.request import Request, urlopen

MANIFEST = Path(__file__).resolve().parents[1] / "data/reference/solar-usno-v1.tsv"
PHENOMENA = {"rise_utc": "Rise", "transit_utc": "Upper Transit", "set_utc": "Set"}
CONTINUOUS_HORIZON = {
    "Object continuously above the Horizon",
    "Object continuously below the Horizon",
}


def fetch_response(url: str, expected_hash: str) -> tuple[dict, bool]:
    with urlopen(Request(url, headers={"User-Agent": "Salah reference verification"}), timeout=20) as response:
        raw = response.read()
    digest = hashlib.sha256(raw).hexdigest()
    import json

    return json.loads(raw), digest == expected_hash


def dated_events(payload: dict, offset_minutes: int) -> tuple[set[str], dict[str, list[str]]]:
    data = payload["properties"]["data"]
    if data["isdst"]:
        raise ValueError("USNO response applied DST unexpectedly")
    if round(float(data["tz"]) * 60) != offset_minutes:
        raise ValueError("USNO response offset differs from manifest")
    local_date = f"{data['year']:04}-{data['month']:02}-{data['day']:02}"
    flags = set()
    events: dict[str, list[str]] = {}
    for item in data["sundata"]:
        phen = item["phen"]
        if item["time"] is None:
            flags.add(phen)
            continue
        local = datetime.fromisoformat(f"{local_date}T{item['time']}:00")
        utc = local - timedelta(minutes=offset_minutes)
        events.setdefault(phen, []).append(utc.strftime("%Y-%m-%dT%H:%M:%SZ"))
    return flags, events


def verify_case(case: dict[str, str]) -> list[str]:
    problems = []
    offset = int(case["utc_offset_minutes"])
    urls = [(case["primary_url"], case["primary_sha256"])]
    if case["secondary_url"] != "-":
        urls.append((case["secondary_url"], case["secondary_sha256"]))
    all_events: dict[str, list[str]] = {}
    primary_flags: set[str] = set()
    for index, (url, expected_hash) in enumerate(urls):
        payload, hash_matches = fetch_response(url, expected_hash)
        if not hash_matches:
            problems.append(f"response {index + 1} SHA-256 changed")
        if payload["apiversion"] != case["source_version"]:
            problems.append(f"response {index + 1} API version changed")
        query = parse_qs(urlparse(url).query)
        data = payload["properties"]["data"]
        returned_date = f"{data['year']:04}-{data['month']:02}-{data['day']:02}"
        if returned_date != query["date"][0]:
            problems.append(f"response {index + 1} date differs from request")
        if index == 0 and returned_date != case["local_date"]:
            problems.append("primary response date differs from case")
        returned_lon, returned_lat = payload["geometry"]["coordinates"]
        if abs(returned_lat - float(case["latitude_deg"])) > 1e-6 or abs(
            returned_lon - float(case["longitude_deg"])
        ) > 1e-6:
            problems.append(f"response {index + 1} coordinates differ from manifest")
        flags, events = dated_events(payload, offset)
        if index == 0:
            primary_flags = flags
        for name, values in events.items():
            all_events.setdefault(name, []).extend(values)

    for field, phen in PHENOMENA.items():
        expected = case[field]
        observed = all_events.get(phen, [])
        if expected == "no_event":
            if not (primary_flags & CONTINUOUS_HORIZON) or observed:
                problems.append(f"{field}: no-event marker no longer matches source")
        elif expected == "unreported":
            if observed:
                problems.append(f"{field}: source now reports {observed}")
        elif expected not in observed:
            problems.append(f"{field}: expected {expected}, source reports {observed}")
    return problems


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--case", action="append", default=[], help="stable case ID; repeatable")
    parser.add_argument("--all", action="store_true", help="verify all manifest cases")
    args = parser.parse_args()
    if args.all == bool(args.case):
        parser.error("choose --all or at least one --case")

    with MANIFEST.open(newline="") as handle:
        cases = list(csv.DictReader(handle, delimiter="\t"))
    selected = cases if args.all else [case for case in cases if case["case_id"] in args.case]
    if len(selected) != (len(cases) if args.all else len(set(args.case))):
        parser.error("one or more case IDs were not found in the manifest")

    failures = 0
    for case in selected:
        try:
            problems = verify_case(case)
        except Exception as error:
            problems = [f"source retrieval/format error: {error}"]
        if problems:
            failures += 1
            print(f"{case['case_id']}: REVIEW — {'; '.join(problems)}")
        else:
            print(f"{case['case_id']}: source hash, metadata, and selected events match")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
