package main

import (
	"fmt"
	"github.com/Erchard/osanwe/osanwenet/pb"
	"github.com/Erchard/osanwe/osanwenet/service"
	"google.golang.org/grpc"
	"log"
	"net"
)

func main() {

	helloServer := service.NewOsanweServer()
	grpcServer := grpc.NewServer()
	pb.RegisterHelloServiceServer(grpcServer, helloServer)

	listener, err := net.Listen("tcp", "0.0.0.0:8080")
	if err != nil {
		log.Fatal(err)
	}
	fmt.Printf("Server started %v \n", listener.Addr())
	err = grpcServer.Serve(listener)
	if err != nil {
		log.Fatal(err)
	}

}
