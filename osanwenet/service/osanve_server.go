package service

import (
	"context"
	"fmt"
	"github.com/Erchard/osanwe/osanwenet/pb"
	"google.golang.org/grpc/peer"
)

type OsanweServer struct {
}

func NewOsanweServer() *OsanweServer {
	return &OsanweServer{}
}

func (server *OsanweServer) Hello(ctx context.Context, req *pb.HelloRequest) (*pb.HelloResponse, error) {
	msg := req.GetMsg()
	fmt.Printf("Message from client: %v \n", msg)

	p, ok := peer.FromContext(ctx)
	respmsg := fmt.Sprintf("Client %v %v", ok, p)

	resp := &pb.HelloResponse{
		Msg: respmsg,
	}
	return resp, nil
}
