#!/usr/bin/python3
import csv
import math

with open('many_runs.csv', 'w', newline='') as csvfile:
    fieldnames = ["id", "game", "category", "submitter", "video", "description", "score", "time", "verifier", "status", "created_at", "verified_at", "edited_at", "flags"]
    writer = csv.DictWriter(csvfile, fieldnames=fieldnames)
    writer.writeheader()
    for value in range(1, 8000):
        mins = value % 60
        hours = math.floor(value / 3600)
        row = {
            "id": value,
            "game": 1,
            "category": 1,
            "submitter": 1,
            "video": "https://www.youtube.com/watch?v=vOLivyykLqk",
            "description": f"test run {value}",
            "score": value * 100,
            "time": 500 - value,
            "verifier": None,
            "status": (value % 3) - 1,
            "created_at": f"01-01-2020 {hours:02}:{mins:02}",
            "verified_at": None,
            "edited_at": None,
            "flags": 0
        }
        writer.writerow(row)

        

