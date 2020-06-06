package db

import (
	"fmt"
	"github.com/boltdb/bolt"
	"time"
)

var db *bolt.DB
var MySettings = []byte("MySettings")
var AddressBook = []byte("AddressBook")
var started = false

func start() error {
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
func Required() error {

	var err error
	if started {
		return nil
	} else {
		err = start()
		return err
	}
}
