package book

import (
	"github.com/Erchard/osanwe/auth"
	"github.com/Erchard/osanwe/pb"
)

var pub_key *pb.PubKey

func init() {
	pub_key = auth.GetMyPubkey()

}
