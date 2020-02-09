package network

import (
	"fmt"
	"github.com/Erchard/osanwe/osanwego/protocol"
	listener "github.com/Erchard/osanwe/osanwego/tcp/server"
	"github.com/golang/protobuf/proto"
	"log"
)

func Connect() error {
	fmt.Println("Connecting to network")
	greeting := &protocol.Greeting{
		Version: 1,
		Port:    listener.GetPort(),
		Pubkey:  nil,
	}

	data, err := proto.Marshal(greeting)
	if err != nil {
		log.Fatal("Marshaling error: ", err)
	}

	fmt.Println(data)

	return nil
}
