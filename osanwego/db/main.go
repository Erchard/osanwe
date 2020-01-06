package main

import (
	"fmt"
	"github.com/DanielMorsing/rocksdb"
)

func main() {

	opts := rocksdb.NewOptions()
	opts.SetCache(rocksdb.NewLRUCache(3 << 30))
	opts.SetCreateIfMissing(true)
	db, err := rocksdb.Open("/path/to/db", opts)
	if err != nil {
		fmt.Println(err)
	}

	ro := rocksdb.NewReadOptions()
	wo := rocksdb.NewWriteOptions()

	err = db.Put(wo, []byte("mykey"), []byte("my data"))

	data, err := db.Get(ro, []byte("mykey"))

	err = db.Put(wo, []byte("enotherkey"), data)
	fmt.Printf("\n%s\n", data)

	err = db.Delete(wo, []byte("key"))

}
