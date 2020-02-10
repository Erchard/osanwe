package main

import (
	"fmt"
	"github.com/rapidloop/skv"
)

func main() {

	// open the store
	store, err := skv.Open("sessions.db")
	if err != nil {
		fmt.Errorf("error: %s", err)
	}

	// put: encodes value with gob and updates the boltdb
	err = skv.KVStore{}(sessionId, info)
	if err != nil {
		fmt.Errorf("error: %s", err)
	}

	// get: fetches from boltdb and does gob decode
	err = skv.Get(sessionId, &info)
	if err != nil {
		fmt.Errorf("error: %s", err)
	}

	// delete: seeks in boltdb and deletes the record
	err = skv.Delete(sessionId)
	if err != nil {
		fmt.Errorf("error: %s", err)
	}
	// close the store
	store.Close()
}
