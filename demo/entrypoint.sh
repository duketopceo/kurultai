#!/bin/sh
# Demo image entrypoint: seed the fixture brain in the background, then
# exec the daemon as PID 1 so signals/cleanup behave normally.
/app/demo/seed.sh &
exec kurultai "$@"
