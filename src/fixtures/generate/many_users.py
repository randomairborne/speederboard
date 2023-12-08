#!/usr/bin/python3
import math

password = "$argon2id$v=19$m=16,t=2,p=1$aGNwRU5VY2QzREdkMjBmUw$74WNFkPqrp62SHn33s5MNQ"
email = ".nerd@example.com"

def write_file():
    with open('many_users.sql', 'w', newline='') as f:
        f.write("INSERT INTO users (id, email, username, password, biography, admin, stylesheet, banner, pfp, created_at, language, flags) VALUES")
        for value in range(1, 8000):
            mins = value % 60
            hours = math.floor(value / 3600)
            f.write(f"({value}, '{value}{email}', 'test user {value}', '{password}', 'this is test user {value}', false, false, false, false, '01-01-2020 {hours:02}:{mins:02}', 'en', 0)")
            if value == 7999:
                f.write(";")
            else:
                f.write(",")


if __name__ == "__main__":
    write_file()

        

