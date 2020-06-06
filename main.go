package main

import (
	"fmt"
	"github.com/Erchard/osanwe/communicator"
)

func main() {
	fmt.Println("Osanwe 0.3")
	err := communicator.Required()
	if err != nil {
		fmt.Println(err)
	} else {
		fmt.Println("Osanwe started")
	}

}
