package main

import (
	"fmt"
	"github.com/golang/protobuf/proto"
	"log"
)

func main() {
	fmt.Println("Hello World!")

	elliot := &Person{
		Name: "Elliot",
		Age:  24,
		SocialFollowers: &SocialFollowers{
			Youtube: 1400,
			Twitter: 2500,
		},
	}

	data, err := proto.Marshal(elliot)
	if err != nil {
		log.Fatal("Marshaling error: ", err)
	}

	fmt.Println(data)

	newElliot := &Person{}
	err = proto.Unmarshal(data, newElliot)
	if err != nil {
		log.Fatal("Unmarshaling error: ", err)
	}

	fmt.Println(newElliot.GetAge())
	fmt.Println(newElliot.GetName())
	fmt.Println(newElliot.GetSocialFollowers().GetTwitter())
	fmt.Println(newElliot.GetSocialFollowers().GetYoutube())

}
