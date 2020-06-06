package listener

import (
	"fmt"
	"github.com/Erchard/osanwe/db/mynode"
	"google.golang.org/grpc"
	"log"
	"net"
)

func Start() error {

	//greetingServer := service.NewGreetingServer()
	grpcServer := grpc.NewServer()
	//pb.RegisterGreetingServiceServer(grpcServer, greetingServer)

	listener, err := listen()
	if err != nil {
		return err
	}

	var port int = int(mynode.GetPort())

	if listener.Addr().(*net.TCPAddr).Port != port {
		fmt.Printf("port: %v \n", port)
		fmt.Printf("listener.Addr().(*net.TCPAddr).Port: %v \n", listener.Addr().(*net.TCPAddr).Port)
		mynode.SaveNewPort(int32(listener.Addr().(*net.TCPAddr).Port))
	}

	fmt.Println("Listener start: " + listener.Addr().String())

	go acceptConnection(listener, grpcServer)

	return nil
}

func listen() (net.Listener, error) {
	var ipaddresses [][]byte = mynode.GeIPAdresses()
	var port int = int(mynode.GetPort())
	var err error
	for i, ipaddress := range ipaddresses {
		laddr := net.TCPAddr{IP: ipaddress, Port: port} // Port == 0 - free port
		addrString := laddr.String()
		fmt.Println(addrString)

		listener, err := net.Listen("tcp", addrString)
		if err != nil {
			log.Printf("%n Address %s not available. \n%s\n", i, laddr, err)
		} else {
			return listener, nil
		}
	}
	return nil, err
}

func acceptConnection(listener net.Listener, grpcServer *grpc.Server) {
	err := grpcServer.Serve(listener)
	if err != nil {
		log.Fatal(err)
	}
}
