package service

import (
	"context"
	"fmt"
	"github.com/Erchard/osanwe/osanwenet/pb"
)

type OsanweServer struct {
}

func NewOsanweServer() *OsanweServer {
	return &OsanweServer{}
}

func (server *OsanweServer) Hello(ctx context.Context, req *pb.HelloRequest) (*pb.HelloResponse, error) {
	msg := req.GetMsg()
	fmt.Printf("Message from client: %v \n", msg)

	resp := &pb.HelloResponse{
		Msg: "Server is ok!",
	}
	return resp, nil
}
