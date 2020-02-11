// Code generated by protoc-gen-go. DO NOT EDIT.
// source: node.proto

package protocol

import (
	fmt "fmt"
	proto "github.com/golang/protobuf/proto"
	math "math"
)

// Reference imports to suppress errors if they are not otherwise used.
var _ = proto.Marshal
var _ = fmt.Errorf
var _ = math.Inf

// This is a compile-time assertion to ensure that this generated file
// is compatible with the proto package it is being compiled against.
// A compilation error at this line likely means your copy of the
// proto package needs to be updated.
const _ = proto.ProtoPackageIsVersion3 // please upgrade the proto package

type Node struct {
	Pubkey               *PubKey  `protobuf:"bytes,1,opt,name=pubkey,proto3" json:"pubkey,omitempty"`
	Ipaddresses          [][]byte `protobuf:"bytes,2,rep,name=ipaddresses,proto3" json:"ipaddresses,omitempty"`
	Port                 int32    `protobuf:"varint,3,opt,name=port,proto3" json:"port,omitempty"`
	XXX_NoUnkeyedLiteral struct{} `json:"-"`
	XXX_unrecognized     []byte   `json:"-"`
	XXX_sizecache        int32    `json:"-"`
}

func (m *Node) Reset()         { *m = Node{} }
func (m *Node) String() string { return proto.CompactTextString(m) }
func (*Node) ProtoMessage()    {}
func (*Node) Descriptor() ([]byte, []int) {
	return fileDescriptor_0c843d59d2d938e7, []int{0}
}

func (m *Node) XXX_Unmarshal(b []byte) error {
	return xxx_messageInfo_Node.Unmarshal(m, b)
}
func (m *Node) XXX_Marshal(b []byte, deterministic bool) ([]byte, error) {
	return xxx_messageInfo_Node.Marshal(b, m, deterministic)
}
func (m *Node) XXX_Merge(src proto.Message) {
	xxx_messageInfo_Node.Merge(m, src)
}
func (m *Node) XXX_Size() int {
	return xxx_messageInfo_Node.Size(m)
}
func (m *Node) XXX_DiscardUnknown() {
	xxx_messageInfo_Node.DiscardUnknown(m)
}

var xxx_messageInfo_Node proto.InternalMessageInfo

func (m *Node) GetPubkey() *PubKey {
	if m != nil {
		return m.Pubkey
	}
	return nil
}

func (m *Node) GetIpaddresses() [][]byte {
	if m != nil {
		return m.Ipaddresses
	}
	return nil
}

func (m *Node) GetPort() int32 {
	if m != nil {
		return m.Port
	}
	return 0
}

func init() {
	proto.RegisterType((*Node)(nil), "protocol.Node")
}

func init() { proto.RegisterFile("node.proto", fileDescriptor_0c843d59d2d938e7) }

var fileDescriptor_0c843d59d2d938e7 = []byte{
	// 133 bytes of a gzipped FileDescriptorProto
	0x1f, 0x8b, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0xff, 0xe2, 0xe2, 0xca, 0xcb, 0x4f, 0x49,
	0xd5, 0x2b, 0x28, 0xca, 0x2f, 0xc9, 0x17, 0xe2, 0x00, 0x53, 0xc9, 0xf9, 0x39, 0x52, 0x3c, 0x05,
	0xa5, 0x49, 0xd9, 0xa9, 0x95, 0x10, 0x71, 0xa5, 0x34, 0x2e, 0x16, 0xbf, 0xfc, 0x94, 0x54, 0x21,
	0x0d, 0x2e, 0x36, 0x88, 0xb8, 0x04, 0xa3, 0x02, 0xa3, 0x06, 0xb7, 0x91, 0x80, 0x1e, 0x4c, 0x83,
	0x5e, 0x40, 0x69, 0x92, 0x77, 0x6a, 0x65, 0x10, 0x54, 0x5e, 0x48, 0x81, 0x8b, 0x3b, 0xb3, 0x20,
	0x31, 0x25, 0xa5, 0x28, 0xb5, 0xb8, 0x38, 0xb5, 0x58, 0x82, 0x49, 0x81, 0x59, 0x83, 0x27, 0x08,
	0x59, 0x48, 0x48, 0x88, 0x8b, 0xa5, 0x20, 0xbf, 0xa8, 0x44, 0x82, 0x59, 0x81, 0x51, 0x83, 0x35,
	0x08, 0xcc, 0x4e, 0x62, 0x03, 0x1b, 0x67, 0x0c, 0x08, 0x00, 0x00, 0xff, 0xff, 0xd6, 0x98, 0xb5,
	0xa4, 0x94, 0x00, 0x00, 0x00,
}
