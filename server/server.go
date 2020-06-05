package server

import (
	"github.com/Erchard/osanwe/book"
	"github.com/Erchard/osanwe/db"
)

var Started = false

func Start() error {

	var err error
	err = db.Start()

	Started = true
	err = book.Start()
	return err
}
