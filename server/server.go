package server

import "github.com/Erchard/osanwe/book"

var Started = false

func Start() {
	Started = true
	book.Start()
}
