package book

import (
	auth "github.com/Erchard/osanwe/auth"
	pb "github.com/Erchard/osanwe/pb"
)

var pub_key *pb.PubKey

func init() {
	pub_key = auth.GetMyPubkey()

}
