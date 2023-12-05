#!/usr/bin/python3
import math

youtube = "https://www.youtube.com/watch?v=vOLivyykLqk"

def write_file():
    with open('many_runs.sql', 'w', newline='') as f:
        f.write("INSERT INTO runs (id, game, category, submitter, video, description, score, time, verifier, status, created_at, verified_at, edited_at, flags) VALUES")
        for value in range(1, 8000):
            mins = value % 60
            hours = math.floor(value / 3600)
            f.write(f"({value}, 1, 1, 1, '{youtube}', 'test run {value}', {value * 500}, {500 - value}, NULL, {(value % 3) - 1}, '01-01-2020 {hours:02}:{mins:02}', NULL, NULL, 0)")
            if value == 7999:
                f.write(";")
            else:
                f.write(",")


if __name__ == "__main__":
    write_file()

        

