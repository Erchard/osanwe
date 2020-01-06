package signexample

import (
	"crypto/ecdsa"
	"crypto/elliptic"
	"crypto/md5"
	"crypto/rand"
	"fmt"
	"io"
	"math/big"
)

func main() {
	publicKeyCurve := elliptic.P256()

	privatekey := new(ecdsa.PrivateKey)
	privatekey, err := ecdsa.GenerateKey(publicKeyCurve, rand.Reader)
	if err != nil {
		fmt.Println(err)
	}

	var pubkey ecdsa.PublicKey
	pubkey = privatekey.PublicKey

	fmt.Println("Private Key:")
	fmt.Printf("%x \n", privatekey)

	fmt.Println("Public Key:")
	fmt.Printf("%x \n", pubkey)

	h := md5.New()
	r := big.NewInt(0)
	s := big.NewInt(0)

	io.WriteString(h, "This is a message to be signed and verified by ECDSA!")
	signhash := h.Sum(nil)

	r, s, serr := ecdsa.Sign(rand.Reader, privatekey, signhash)
	if serr != nil {
		fmt.Println(serr)
	}

	signature := r.Bytes()

	signature = append(signature, s.Bytes()...)

	fmt.Printf("signature: %x\n", signature)

	verifystatus := ecdsa.Verify(&pubkey, signhash, r, s)
	fmt.Println(verifystatus)
}
