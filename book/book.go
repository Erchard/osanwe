package book

import (
	"fmt"
	"github.com/Erchard/osanwe/db"
	"github.com/Erchard/osanwe/mynode"
	"github.com/Erchard/osanwe/pb"
	"github.com/jinzhu/copier"
	"sync"
)

var pub_key *pb.PubKey
var started = false
var inmemoryNodes InMemoryNodes

type InMemoryNodes struct {
	mutex sync.RWMutex
	byid  map[[32]byte]*pb.Node
	byIp  map[[4]byte]*pb.Node
}

func start() error {
	var err error
	db.Required()
	inmemoryNodes = *NewInMemoryNodes()
	fillDataFromDB()
	fmt.Printf("Book started with %d nodes\n", len(inmemoryNodes.byid))
	return err
}

func Required() error {

	var err error
	if started {
		return nil
	} else {
		err = start()
		started = true
		return err
	}
}

func NewInMemoryNodes() *InMemoryNodes {

	store := &InMemoryNodes{
		byid: make(map[[32]byte]*pb.Node),
		byIp: make(map[[4]byte]*pb.Node),
	}
	// fill data from bolt db

	return store
}

func fillDataFromDB() {
	nodeList := db.GetAllNodes()

	if len(nodeList) == 0 {
		addSeedNode()
	} else {

		for _, node := range nodeList {
			addNodeToMemory(node)
		}
	}
}

func addSeedNode() {
	fmt.Println("Add seed Node")

	seedNode := mynode.CreateSeedNode("5.187.6.75:12345",
		"c94b3097970d59fac8ac565c3ee5dfbb0764258f3a1b97f9c18c64fb191f9c62",
		"c94b3097970d59fac8ac565c3ee5dfbb0764258f3a1b97f9c18c64fb191f9c62")
	addNodeToMemory(seedNode)

	db.SaveNode(seedNode)

	var nodelist []*pb.Node = db.GetAllNodes()
	fmt.Println(len(nodelist))
	for i, node := range nodelist {
		fmt.Printf("%v Node: %x \n", i, node.Pubkey.X)
	}

}

func addNodeToMemory(node *pb.Node) {
	var id [32]byte
	copy(id[:], node.Id[:32])
	inmemoryNodes.byid[id] = node

	for _, ipaddress := range node.Ipaddresses {
		var ip [4]byte
		copy(ip[:], ipaddress[:4])
		inmemoryNodes.byIp[ip] = node
	}
}

func (store *InMemoryNodes) Save(node *pb.Node) error {
	store.mutex.Lock()
	defer store.mutex.Unlock()

	other := &pb.Node{}
	err := copier.Copy(other, node)
	if err != nil {
		fmt.Println(err)
		return err
	}
	addNodeToMemory(node)
	go db.SaveNode(node)

	return nil
}
