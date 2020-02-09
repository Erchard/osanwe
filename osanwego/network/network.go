package network

import (
	"fmt"
	"github.com/Erchard/osanwe/osanwego/protocol"
	"github.com/golang/protobuf/proto"
	"log"
)

func Connect() error {
	fmt.Println("Connecting to network")
	greeting := &protocol.Greeting{
		Version: 0,
		Port:    0,
		Pubkey:  nil,
	}

	data, err := proto.Marshal(greeting)
	if err != nil {
		log.Fatal("Marshaling error: ", err)
	}

	fmt.Println(data)

	return nil
}
