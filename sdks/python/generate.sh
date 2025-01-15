#!/bin/sh

OUT_DIR=$1

# If there is no argument passed, use the default output directory "protocol"

if [ -z "$OUT_DIR" ]; then
	OUT_DIR="protocol"
fi

cd hyveos_sdk

poetry run python -m grpc_tools.protoc -I ../../../protos --python_out=$OUT_DIR --pyi_out=$OUT_DIR --grpc_python_out=$OUT_DIR ../../../protos/bridge.proto

../fix_grpc_imports.sh $OUT_DIR
