package main

import (
	"fmt"
	"github.com/boltdb/bolt"
)

func main() {

	db, err := bolt.Open("my.db", 0600, nil)
	if err != nil {
		fmt.Println("Error")
	}

	db.Update(func(tx *bolt.Tx) error {
		b := tx.Bucket([]byte("MyBucket"))
		err := b.Put([]byte("answer"), []byte("42"))
		return err
	})

	db.View(func(tx *bolt.Tx) error {
		b := tx.Bucket([]byte("MyBucket"))
		v := b.Get([]byte("answer"))
		fmt.Printf("The answer is: %s\n", v)
		return nil
	})

	defer db.Close()

}
