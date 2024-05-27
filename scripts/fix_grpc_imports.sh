#!/usr/bin/env bash
DIR=$1

if [ -z "$DIR" ]; then
  echo "Usage: $0 <directory>"
  exit 1
fi

echo "Fixing gRPC imports in $DIR"

for file in $(find "$DIR" -name '*_pb2_grpc.py'); do
  echo "Fixing $file"
  sed -i "$file" -e 's/^import \(.*_pb2\)/from . import \1/'
done
