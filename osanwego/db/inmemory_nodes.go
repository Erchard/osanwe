package db

import (
	"fmt"
	"github.com/Erchard/osanwe/pb"
	"github.com/jinzhu/copier"
	"sync"
)

type InMemoryNodes struct {
	mutex sync.RWMutex
	byid  map[[32]byte]*pb.Node
	byIp  map[[4]byte]*pb.Node
}

func NewInMamoryNodes() *InMemoryNodes {

	store := &InMemoryNodes{
		byid: make(map[[32]byte]*pb.Node),
		byIp: make(map[[4]byte]*pb.Node),
	}
	// fill data from bolt db
	return store
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

	return nil
}
