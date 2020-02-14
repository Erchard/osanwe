package main

import (
	"fmt"
	"github.com/Erchard/osanwe/osanwego/db"
	"github.com/Erchard/osanwe/osanwego/network"
	"github.com/Erchard/osanwe/osanwego/nodekeys"
	"github.com/Erchard/osanwe/osanwego/protocol"
	"github.com/Erchard/osanwe/osanwego/tcp/server"
	"os"
	"time"
)

func main() {
	fmt.Println("Starting Osanwe")
	must(db.Init())
	must(nodekeys.Restore())
	must(listener.Start())
	TestNode()
	must(network.Connect())

	time.Sleep(5 * time.Second)
	fmt.Println("All done!")
}

func must(err error) {
	if err != nil {
		fmt.Println(err.Error())
		os.Exit(1)
	}
}

func TestNode() {

	x, y := nodekeys.CreateTestKey()
	ipindb := []byte{192, 168, 1, 201}

	myNode := &protocol.Node{
		Pubkey: &protocol.PubKey{
			X: x,
			Y: y,
		},
		Ipaddresses: [][]byte{ipindb},
		Port:        8080,
	}

	db.SaveNode(myNode)

	var nodelist []*protocol.Node = db.GetAllNodes()
	fmt.Println(len(nodelist))
	for i, node := range nodelist {
		fmt.Printf("%v Node: %x \n", i, node.Pubkey.X)
	}

}
