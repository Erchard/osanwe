package network

import (
	"fmt"
	"github.com/Erchard/osanwe/osanwego/db"
	"github.com/Erchard/osanwe/osanwego/nodekeys"
	"github.com/Erchard/osanwe/osanwego/protocol"
	"github.com/Erchard/osanwe/osanwego/tcp/client"
	listener "github.com/Erchard/osanwe/osanwego/tcp/server"
	"github.com/golang/protobuf/proto"
	"log"
	"net"
)

func Connect() error {
	fmt.Println("Connecting to network")
	x, y := nodekeys.GetPubKey()
	greeting := &protocol.Greeting{
		Version: 0,
		Port:    listener.GetPort(),
		Pubkey: &protocol.PubKey{
			X: x,
			Y: y,
		},
	}

	data, err := proto.Marshal(greeting)
	if err != nil {
		log.Fatal("Marshaling error: ", err)
	}

	fmt.Println(data)

	var nodelist []*protocol.Node = db.GetAllNodes()
	fmt.Println(len(nodelist))
	for i, node := range nodelist {
		fmt.Printf("%v Node: %x \n", i, node.Pubkey.X)
		ipindb := node.Ipaddresses[0]
		a := ipindb[0]
		b := ipindb[1]
		c := ipindb[2]
		d := ipindb[3]
		ipaddress := net.IPv4(a, b, c, d)

		port := int(node.Port)

		laddr := net.TCPAddr{IP: ipaddress, Port: port} // Port == 0 - free port
		addrString := laddr.String()
		client.Connect(addrString)
	}

	return nil
}
