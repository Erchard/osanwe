package main

import (
	"fmt"
	"github.com/Erchard/osanwe/osanwego/db"
	"os"
)

func main() {
	fmt.Println("Starting Osanwe")
	must(db.Init())

}

func must(err error) {
	if err != nil {
		fmt.Println(err.Error())
		os.Exit(1)
	}
}
