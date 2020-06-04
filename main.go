package main

import (
	"fmt"
	"github.com/Erchard/osanwe/server"
)

func main() {
	fmt.Println("Osanwe 0.3")
	server.Start()
	fmt.Println(server.Started)
}
