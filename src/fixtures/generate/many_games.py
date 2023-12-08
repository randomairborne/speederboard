#!/usr/bin/python3
import math

def write_file():
    with open('many_games.sql', 'w', newline='') as f:
        f.write("SET constraints all deferred;")
        f.write("INSERT INTO games (id, name, slug, url, default_category, description, banner, cover_art, flags) VALUES")
        for value in range(1, 8000):
            mins = value % 60
            hours = math.floor(value / 3600)
            f.write(f"({value}, 'test game {value}', 'gameslug{value}', 'https://example.com', 1, 'test game {value}', false, false, 0)")
            if value == 7999:
                f.write(";")
            else:
                f.write(",")
        f.write("INSERT INTO categories (id, game, name, description, rules, scoreboard, flags) VALUES")
        for value in range(1, 8000):
            mins = value % 60
            hours = math.floor(value / 3600)
            f.write(f"({value}, {value}, 'test category {value}', 'this is test category {value}', 'rules for category {value}', true, 0)")
            if value == 7999:
                f.write(";")
            else:
                f.write(",")


if __name__ == "__main__":
    write_file()

        

