package main

import (
	"fmt"
	"github.com/Erchard/osanwe/osanwego/db"
)

func main() {
	fmt.Println("Starting Osanwe")
	db.Init()

}
