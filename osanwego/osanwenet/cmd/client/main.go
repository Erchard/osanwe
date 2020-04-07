package main

import (
	"context"
	"fmt"
	"github.com/Erchard/osanwe/osanwenet/pb"
	"google.golang.org/grpc"
	"log"
)

func main() {
	fmt.Println("Client started")
	conn, err := grpc.Dial("0.0.0.0:8080", grpc.WithInsecure())
	if err != nil {
		log.Fatal(err)
	}

	helloClient := pb.NewHelloServiceClient(conn)

	req := &pb.HelloRequest{
		Msg: "Client is ok!",
	}

	res, err := helloClient.Hello(context.Background(), req)
	if err != nil {
		log.Fatal(err)
	}

	fmt.Printf("Response from server: %v \n", res)

}
