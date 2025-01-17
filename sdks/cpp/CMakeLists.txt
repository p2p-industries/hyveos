cmake_minimum_required(VERSION 3.20)
project(hyveosSDK)

set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

find_package(gRPC CONFIG REQUIRED)
find_package(Protobuf REQUIRED)

set(PROTO_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../../protos")
set(PROTO_FILES "${PROTO_DIR}/script.proto")


include_directories(${CMAKE_CURRENT_BINARY_DIR})

add_library(hyveosSDK main.cpp)
target_link_libraries(hyveosSDK PUBLIC gRPC::grpc++)
protobuf_generate(
	TARGET hyveosSDK
	PROTOS ${PROTO_FILES}
	IMPORT_DIRS ${PROTO_DIR}
	LANGUAGE cpp
)
protobuf_generate(
	TARGET hyveosSDK
	PROTOS ${PROTO_FILES}
	IMPORT_DIRS ${PROTO_DIR}
	LANGUAGE grpc
	GENERATE_EXTENSIONS .grpc.pb.h .grpc.pb.cc
	PLUGIN "protoc-gen-grpc=\$<TARGET_FILE:gRPC::grpc_cpp_plugin>"
)
