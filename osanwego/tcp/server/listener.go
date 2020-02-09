package listener

import (
	"bufio"
	"encoding/binary"
	"fmt"
	"github.com/Erchard/osanwe/osanwego/db"
	"log"
	"net"
)

var portindb = []byte("portindb")

func Start() error {

	var port int = restorePort()

	laddr := net.TCPAddr{IP: net.IPv4(127, 0, 0, 1), Port: port} // Port == 0 - free port
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
		saveNewPort(listener.Addr().(*net.TCPAddr).Port)
	}

	fmt.Println("Listener start: " + listener.Addr().String())

	go acceptConnection(listener)

	return nil
}

func saveNewPort(port int) {
	fmt.Printf("Save new port: %v \n", port)
	bs := make([]byte, 4)
	binary.LittleEndian.PutUint32(bs, uint32(port))
	db.Set(portindb, bs)
}

func restorePort() int {
	bs := db.Get(portindb)
	if bs != nil {
		port := binary.LittleEndian.Uint32(bs)
		return int(port)
	}
	return 0
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
