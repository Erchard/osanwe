# Compile Protocol Buffer

Install `protoc` compiller
```http request
https://github.com/protocolbuffers/protobuf/releases/tag/v3.11.3
```
Install the Go protocol buffers plugin 

```shell script
go get -u github.com/golang/protobuf/protoc-gen-go
```
```shell script
go get google.golang.org/grpc
```

In dir `osanweauth` compile command:

```shell script
	protoc --proto_path=proto proto/*.proto --go_out=plugins=grpc:pb
```

```shell script	
	go get -u  github.com/Erchard/osanwe/osanwego
```

```shell script
protoc -I=./ --go_out=./ *.proto
```