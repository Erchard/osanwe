package mynode

import (
	"crypto/ecdsa"
	"crypto/elliptic"
	"crypto/rand"
	"fmt"
	"github.com/Erchard/osanwe/osanwego/db"
	"math/big"
)

var mynodekeys = []byte("mynodekeys")

var dkeyBytes []byte
var nodekey *ecdsa.PrivateKey

func Restore() error {
	fmt.Println("Restore node keys")
	dkeyBytes = db.GetSettings(mynodekeys)
	if dkeyBytes == nil {
		fmt.Println("Node keys not found. Creating...")
		createKeys()
	} else {
		x, y := elliptic.P256().ScalarBaseMult(dkeyBytes)
		dkey := new(big.Int)
		dkey.SetBytes(dkeyBytes)
		nodekey = &ecdsa.PrivateKey{
			PublicKey: ecdsa.PublicKey{
				Curve: elliptic.P256(),
				X:     x,
				Y:     y,
			},
			D: dkey,
		}
	}
	fmt.Printf("privkey.D: %x \n", dkeyBytes)

	return nil
}

func createKeys() {
	privkey, err := ecdsa.GenerateKey(elliptic.P256(), rand.Reader)
	if err != nil {
		fmt.Println(err.Error())
	}
	fmt.Printf("\n %x \n", privkey)

	nodekey = privkey
	dkeyBytes = privkey.D.Bytes()

	err = db.SetSettings(mynodekeys, dkeyBytes)
	if err != nil {
		fmt.Println(err.Error())
	}
	fmt.Println("Save key to DB")
}

func GetPubKey() ([]byte, []byte) {
	return nodekey.X.Bytes(), nodekey.Y.Bytes()
}

func CreateTestKey() ([]byte, []byte) {
	privkey, err := ecdsa.GenerateKey(elliptic.P256(), rand.Reader)
	if err != nil {
		fmt.Println(err.Error())
	}
	return privkey.X.Bytes(), privkey.Y.Bytes()
}