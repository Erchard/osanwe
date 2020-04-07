package main

import (
	"fmt"
	"github.com/Erchard/osanwe/osanwego/db"
	"github.com/Erchard/osanwe/osanwego/mynode"
	"github.com/Erchard/osanwe/osanwego/network"
	"github.com/Erchard/osanwe/osanwego/protocol"
	listener "github.com/Erchard/osanwe/osanwego/tcp/server"
	"os"
	"time"
)

func main() {
	fmt.Println("Starting Osanwe")
	must(db.Init())
	must(mynode.Restore())
	must(listener.Start())
	AddSeedNode()
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

func AddSeedNode() {
	fmt.Println("Add seed Node")
	x, y := mynode.CreateTestKey()

	ipindb := []byte{5, 187, 6, 75}

	myNode := &protocol.Node{
		Pubkey: &protocol.PubKey{
			X: x,
			Y: y,
		},
		Ipaddresses: [][]byte{ipindb},
		Port:        42647,
	}

	db.SaveNode(myNode)

	var nodelist []*protocol.Node = db.GetAllNodes()
	fmt.Println(len(nodelist))
	for i, node := range nodelist {
		fmt.Printf("%v Node: %x \n", i, node.Pubkey.X)
	}

}
