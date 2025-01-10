#!/bin/sh

cd hyveos_sdk

poetry run python -m grpc_tools.protoc -I ../../../protos --python_out=protocol/ --pyi_out=protocol/ --grpc_python_out=protocol/ ../../../protos/script.proto

../fix_grpc_imports.sh protocol/
