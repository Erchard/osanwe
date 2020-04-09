package service

import (
	"context"
	"fmt"
	"github.com/Erchard/osanwe/osanwego/grpc/client"
	"github.com/Erchard/osanwe/osanwego/pb"
	"google.golang.org/grpc/peer"
	"net"
)

type GreetingServer struct {
}

func NewGreetingServer() *GreetingServer {
	return &GreetingServer{}
}

func (server *GreetingServer) Greeting(ctx context.Context, req *pb.GreetingRequest) (*pb.GreetingResponse, error) {

	fmt.Printf("Client port: %d \n", req.Port)
	fmt.Printf("Pubkey client: %x %x \n", req.GetPubkey().GetX(), req.GetPubkey().GetY())

	p, ok := peer.FromContext(ctx)

	ipRemote, portRemote := parseAddr(p.Addr)
	/// :)
	client.Connect(net.TCPAddr{
		IP:   ipRemote,
		Port: int(req.Port),
	}.String())
	///
	respmsg := fmt.Sprintf("Client %v %v", ok, p.Addr)
	fmt.Println(respmsg)
	resp := &pb.GreetingResponse{
		Ipaddresses: ipRemote,
		Port:        portRemote,
		Visible:     true,
	}

	return resp, nil
}

func parseAddr(remoteAddr net.Addr) (net.IP, int32) {

	switch addr := remoteAddr.(type) {
	case *net.UDPAddr:
		return addr.IP, int32(addr.Port)
	case *net.TCPAddr:
		return addr.IP, int32(addr.Port)
	}
	return nil, 0
}
