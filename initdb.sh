#! /bin/bash

# non-interactively create a file with an empty SQLite database
sqlite3 ./cowchat.db "VACUUM;"
