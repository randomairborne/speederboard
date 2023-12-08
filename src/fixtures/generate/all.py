#!/usr/bin/python3
import os
import many_runs
import many_games
import many_users

os.chdir(os.getcwd() + "/src/fixtures/")

many_users.write_file()
many_games.write_file()
many_runs.write_file()