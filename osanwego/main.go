package main

import (
	"fmt"
	"github.com/Erchard/osanwe/osanwego/db"
	"github.com/Erchard/osanwe/osanwego/nodekeys"
	"github.com/Erchard/osanwe/osanwego/tcp/server"
	"os"
)

func main() {
	fmt.Println("Starting Osanwe")
	must(db.Init())
	must(nodekeys.Restore())
	must(listener.Start())

	fmt.Println("All done!")
}

func must(err error) {
	if err != nil {
		fmt.Println(err.Error())
		os.Exit(1)
	}
}
