# Compile Protocol Buffer

Install `protoc` compiller
```http request
https://github.com/protocolbuffers/protobuf/releases/tag/v3.11.3
```
Install the Go protocol buffers plugin 

```shell script
go get -u github.com/golang/protobuf/protoc-gen-go
```
In dir `osanwego/protocol` compile command:
```shell script
protoc -I=./ --go_out=./ *.proto
```