package communicator

import (
	"github.com/Erchard/osanwe/book"
	"github.com/Erchard/osanwe/communicator/grpc/listener"
	"github.com/Erchard/osanwe/db"
	"github.com/Erchard/osanwe/mynode"
)

var started = false

func start() error {

	var err error
	err = db.Required()
	err = mynode.Required()
	listener.Start()
	err = book.Required()
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
