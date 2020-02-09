package db

import (
	"fmt"
	"github.com/boltdb/bolt"
	"time"
)

var MyBucket = []byte("MyBucket")

var db *bolt.DB

func Init() error {

	var err error
	db, err = bolt.Open("osanwe.db", 0600, &bolt.Options{Timeout: 1 * time.Second})
	if err != nil {
		return err
	}
	fmt.Println("Open DB")
	return db.Update(func(tx *bolt.Tx) error {
		_, err = tx.CreateBucketIfNotExists(MyBucket)
		return err
	})
}

func Set(key []byte, val []byte) error {
	return db.Update(func(tx *bolt.Tx) error {
		b := tx.Bucket(MyBucket)
		return b.Put(key, val)
	})
}

func Get(key []byte) []byte {
	var val []byte

	err := db.View(func(tx *bolt.Tx) error {
		b := tx.Bucket(MyBucket)
		val = b.Get(key)
		return nil
	})
	if err != nil {
		fmt.Println(err.Error())
		return nil
	}
	return val
}
