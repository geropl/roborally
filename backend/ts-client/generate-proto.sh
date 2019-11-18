#!/bin/bash

OUT_DIR=$1

protoc ../proto/* -I=../proto -I /usr/lib/protoc/include --plugin=protoc-gen-grpc=`which grpc_tools_node_protoc_plugin` --js_out=import_style=commonjs:$OUT_DIR --grpc-web_out=import_style=typescript,mode=grpcwebtext:$OUT_DIR
protoc ../proto/* -I=../proto -I /usr/lib/protoc/include --plugin=protoc-gen-ts=`which protoc-gen-ts` --ts_out=$OUT_DIR