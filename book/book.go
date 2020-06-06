package book

import (
	"fmt"
	"github.com/Erchard/osanwe/pb"
)

var pub_key *pb.PubKey
var started = false

func start() error {
	var err error
	fmt.Println("Book started")
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
