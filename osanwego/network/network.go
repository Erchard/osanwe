package network

import (
	"fmt"
	"github.com/Erchard/osanwe/osanwego/nodekeys"
	"github.com/Erchard/osanwe/osanwego/protocol"
	"github.com/golang/protobuf/proto"
	"log"
)

func Connect() error {
	fmt.Println("Connecting to network")
	x, y := nodekeys.GetPubKey()
	greeting := &protocol.Greeting{
		Version: 0,
		Port:    0,
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

	return nil
}
