gen-server:
	protoc --rust_out=rs --grpc_out=rs --plugin=protoc-gen-grpc=`which grpc_rust_plugin` *.proto

gen-client:
	protoc --js_out=import_style=commonjs,binary:js --grpc_out=js --plugin=protoc-gen-grpc=`which grpc_tools_node_protoc_plugin` *.proto
