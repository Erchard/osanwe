package book

import (
	"github.com/Erchard/osanwe/osanweauth/auth"
	"github.com/Erchard/osanwe/osanweauth/pb"
)

var pub_key *pb.PubKey

func init() {
	pub_key = auth.GetMyPubkey()

}
