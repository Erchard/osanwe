package osanwego

import (
	"fmt"
	"github.com/Erchard/osanwe/db"
)

func main() {
	fmt.Println("Starting Osanwe")
	db.Init()

}
