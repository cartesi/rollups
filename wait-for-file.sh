#!/bin/sh

set -e

waitFile="$1"
shift
cmd="$@"

until test -e $waitFile
do
  >&2 echo "Waiting for file [$waitFile]."
  sleep 1
done

>&2 echo "Found file [$waitFile]."
exec $cmd
