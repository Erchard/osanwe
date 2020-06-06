package communicator

import (
	"github.com/Erchard/osanwe/book"
	"github.com/Erchard/osanwe/db"
)

var started = false

func start() error {

	var err error
	err = db.Required()

	err = book.Required()
	return err
}

func Required() error {

	var err error
	if started {
		return nil
	} else {
		err = start()
		return err
	}
}
