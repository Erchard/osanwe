package service

import (
	"context"
	"fmt"
	"github.com/Erchard/osanwe/osanwego/pb"
	"google.golang.org/grpc/peer"
)

type GreetingServer struct {
}

func NewGreetingServer() *GreetingServer {
	return &GreetingServer{}
}

func (server *GreetingServer) Greeting(ctx context.Context, req *pb.GreetingRequest) (*pb.GreetingResponse, error) {
	pubkey := req.GetPubkey()
	fmt.Printf("Pubkey client: %v \n", pubkey)

	p, ok := peer.FromContext(ctx)
	respmsg := fmt.Sprintf("Client %v %v", ok, p)
	fmt.Println(respmsg)
	resp := &pb.GreetingResponse{
		Ipaddresses: nil,
		Port:        0,
		Visible:     true,
	}

	return resp, nil
}
