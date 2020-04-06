package service

import (
	"context"
	"github.com/Erchard/osanwe/osanwenet/pb"
)

type OsanweServer struct {
}

func NewOsanweServer() *OsanweServer {
	return &OsanweServer{}
}

func (server *OsanweServer) Hello(ctx context.Context, req *pb.HelloRequest) (*pb.HelloResponse, error) {

}
