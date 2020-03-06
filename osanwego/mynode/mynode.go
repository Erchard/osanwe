package mynode

import (
	"crypto/ecdsa"
	"crypto/elliptic"
	"crypto/rand"
	"crypto/sha256"
	"fmt"
	"github.com/Erchard/osanwe/osanwego/db"
	"github.com/Erchard/osanwe/osanwego/protocol"
	"github.com/golang/protobuf/proto"
	"log"
	"math/big"
	"net"
	"time"
)

var mynodekeys = []byte("mynodekeys")
var mynodeindb = []byte("mynodeindb")

var dkeyBytes []byte
var nodekey *ecdsa.PrivateKey

var myNode = &protocol.Node{}

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

	restoreMyNode()

	return nil
}

func restoreMyNode() {

	data := db.GetSettings(mynodeindb)

	if data == nil {
		createNewNode()
	} else {
		err := proto.Unmarshal(data, myNode)
		if err != nil {
			log.Fatal("Unmarshaling error: ", err.Error())
		}
		fmt.Printf("My Node restored: %x \n", myNode)
	}

}

func createNewNode() {
	fmt.Println("Create new node....")

	myNode.Ipaddresses = getMyIpAddresses()
	myNode.Port = 0
	myNode.Pubkey = &protocol.PubKey{
		X: nodekey.X.Bytes(),
		Y: nodekey.Y.Bytes(),
	}

	xy := append(myNode.Pubkey.X, myNode.Pubkey.Y...)
	hashNode := sha256.Sum256(xy)
	myNode.Id = hashNode[:]
	myNode.Lastactivity = time.Now().UnixNano()

	data, err := proto.Marshal(myNode)
	if err != nil {
		fmt.Println(err.Error())
	}

	db.SetSettings(mynodeindb, data)
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

// https://play.golang.org/p/BDt3qEQ_2H
func getMyIpAddresses() [][]byte {
	ifaces, err := net.Interfaces()
	if err != nil {
		fmt.Println(err.Error())
	}

	ipAddresses := make([][]byte, 0)

	// handle err
	for _, i := range ifaces {
		addrs, err := i.Addrs()
		if err != nil {
			fmt.Println(err.Error())
		}
		// handle err
		for _, addr := range addrs {
			var ip net.IP
			switch v := addr.(type) {
			case *net.IPNet:
				ip = v.IP
			case *net.IPAddr:
				ip = v.IP
			}
			if ip.To4() != nil && !ip.IsLoopback() {
				fmt.Printf("My IP: %s \n", ip)

				ipAddresses = append(ipAddresses, ip.To4())
			}
		}
	}
	//fmt.Printf("%x\n", ipAddresses)
	return ipAddresses
}
