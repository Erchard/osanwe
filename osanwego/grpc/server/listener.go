package listener

import (
	"bufio"
	"fmt"
	"github.com/Erchard/osanwe/osanwego/mynode"
	"github.com/Erchard/osanwe/osanwego/pb"
	"github.com/Erchard/osanwe/osanwego/service"
	"google.golang.org/grpc"
	"log"
	"net"
)

func Start() error {

	greetingServer := service.NewGreetingServer()
	grpcServer := grpc.NewServer()
	pb.RegisterGreetingServiceServer(grpcServer, greetingServer)

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

	go acceptConnection(listener)

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

func acceptConnection(listener net.Listener) {
	for {
		fmt.Println("Ready to connect...")
		conn, err := listener.Accept()
		if err != nil {
			log.Fatal("tcp server accept error", err)
		}

		go handleConnection(conn)
	}
}

func handleConnection(conn net.Conn) {
	bufferBytes, err := bufio.NewReader(conn).ReadBytes('\n')

	if err != nil {
		fmt.Println(err)
		log.Println("Client left..")
		conn.Close()
		return
	}

	message := string(bufferBytes)
	clientAddr := conn.RemoteAddr().String()
	response := fmt.Sprintf(message + " from " + clientAddr + "\n")

	log.Println(response)

	conn.Write([]byte("you sent: " + response))

	handleConnection(conn)
}
