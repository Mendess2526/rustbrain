#!/bin/sh
tr -dc "><+-." > /dev/random | head -c$(($1 * 1000 * 1000 * 1000))
