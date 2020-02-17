package network

import (
	"fmt"
	"github.com/Erchard/osanwe/osanwego/nodekeys"
	"github.com/Erchard/osanwe/osanwego/protocol"
	listener "github.com/Erchard/osanwe/osanwego/tcp/server"
	"github.com/golang/protobuf/proto"
	"log"
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

	client.Connect("192.168.0.105:8080")

	return nil
}
