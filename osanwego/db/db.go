package db

import (
	"crypto/sha256"
	"fmt"
	"github.com/Erchard/osanwe/osanwego/protocol"
	"github.com/boltdb/bolt"
	"github.com/golang/protobuf/proto"

	"time"
)

var MySettings = []byte("MySettings")
var AddressBook = []byte("AddressBook")

var db *bolt.DB

func Init() error {

	var err error
	db, err = bolt.Open("osanwe.db", 0600, &bolt.Options{Timeout: 1 * time.Second})
	if err != nil {
		return err
	}
	fmt.Println("Open DB")
	return db.Update(func(tx *bolt.Tx) error {
		_, err = tx.CreateBucketIfNotExists(MySettings)
		_, err = tx.CreateBucketIfNotExists(AddressBook)
		return err
	})
}

func set(bucket []byte, key []byte, val []byte) error {
	return db.Update(func(tx *bolt.Tx) error {
		b := tx.Bucket(bucket)
		return b.Put(key, val)
	})
}

func get(bucket []byte, key []byte) []byte {
	var val []byte

	err := db.View(func(tx *bolt.Tx) error {
		b := tx.Bucket(bucket)
		val = b.Get(key)
		return nil
	})
	if err != nil {
		fmt.Println(err.Error())
		return nil
	}
	return val
}

func SetSettings(key []byte, val []byte) error {
	return set(MySettings, key, val)
}

func GetSettings(key []byte) []byte {
	return get(MySettings, key)
}

func SaveNode(node *protocol.Node) error {

	xy := append(node.Pubkey.X, node.Pubkey.Y...)
	fmt.Printf("PubKey X: %x \n", node.Pubkey.X)
	fmt.Printf("PubKey Y: %x \n", node.Pubkey.Y)
	fmt.Printf("X+Y: %x \n", xy)

	hashNode := sha256.Sum256(xy)
	fmt.Printf("SHA256: %x \n", hashNode)

	nodeBytes, err := proto.Marshal(node)
	if err != nil {
		fmt.Println(err.Error())
	}
	return set(AddressBook, hashNode[:], nodeBytes)
}

func GetAllNodes() []*protocol.Node {

	nodelist := []*protocol.Node{}

	err := db.View(func(tx *bolt.Tx) error {
		b := tx.Bucket(AddressBook)
		c := b.Cursor()

		for k, v := c.First(); k != nil; k, v = c.Next() {
			node := &protocol.Node{}
			fmt.Printf("key=%x, value=%x \n", k, v)
			err := proto.Unmarshal(v, node)
			if err != nil {
				fmt.Println(err.Error())
			}

			nodelist = append(nodelist, node)

		}
		return nil
	})
	if err != nil {
		fmt.Println(err.Error())
	}

	return nodelist
}
