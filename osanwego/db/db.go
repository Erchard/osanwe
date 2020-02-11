package db

import (
	"fmt"
	"github.com/boltdb/bolt"
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
