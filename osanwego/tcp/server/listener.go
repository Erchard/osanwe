package listener

import (
	"bufio"
	"fmt"
	"github.com/Erchard/osanwe/osanwego/mynode"
	"log"
	"net"
)

func Start() error {

	var ipaddress net.IP = mynode.GetMainIP()
	var port int = int(mynode.GetPort())

	laddr := net.TCPAddr{IP: ipaddress, Port: port} // Port == 0 - free port
	addrString := laddr.String()
	fmt.Println(addrString)
	listener, err := net.Listen("tcp", addrString)
	if err != nil {
		log.Fatal("tcp server listener error:", err)
		return err
	}

	if listener.Addr().(*net.TCPAddr).Port != port {
		fmt.Printf("port: %v \n", port)
		fmt.Printf("listener.Addr().(*net.TCPAddr).Port: %v \n", listener.Addr().(*net.TCPAddr).Port)
		mynode.SaveNewPort(int32(listener.Addr().(*net.TCPAddr).Port))
	}

	fmt.Println("Listener start: " + listener.Addr().String())

	acceptConnection(listener)

	return nil
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
