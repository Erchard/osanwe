package nodekeys

import (
	"crypto/ecdsa"
	"crypto/elliptic"
	"crypto/rand"
	"fmt"
	"github.com/Erchard/osanwe/osanwego/db"
)

var keysindb = []byte("keysindb")

var dkey []byte

func Restore() error {
	fmt.Println("Restore node keys")
	dkey = db.Get(keysindb)
	fmt.Printf("privkey.D: %x", dkey)
	return nil
}

func main() {

	privkey, _ := ecdsa.GenerateKey(elliptic.P256(), rand.Reader)
	fmt.Printf("\n %x \n", privkey)

	dkey := privkey.D

	fmt.Printf("\n %x \n", dkey)

	x, y := elliptic.P256().ScalarBaseMult(dkey.Bytes())

	newkey := &ecdsa.PrivateKey{
		PublicKey: ecdsa.PublicKey{
			Curve: elliptic.P256(),
			X:     x,
			Y:     y,
		},
		D: dkey,
	}

	fmt.Printf("\n %x \n", newkey)

}
