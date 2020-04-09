package client

import (
	"context"
	"fmt"
	"github.com/Erchard/osanwe/osanwego/mynode"
	"github.com/Erchard/osanwe/osanwego/pb"
	"google.golang.org/grpc"
	"log"
)

func Connect(address string) {

	fmt.Println("Client started")
	conn, err := grpc.Dial(address, grpc.WithInsecure())
	if err != nil {
		log.Fatal(err)
	}

	greetingClient := pb.NewGreetingServiceClient(conn)

	x, y := mynode.GetPubKey()

	req := &pb.GreetingRequest{
		Version: 1,
		Port:    mynode.GetPort(),
		Pubkey: &pb.PubKey{
			X: x,
			Y: y,
		},
	}

	res, err := greetingClient.Greeting(context.Background(), req)
	if err != nil {
		log.Fatal(err)
	}

	fmt.Printf("Response from server: %v \n", res)
}
