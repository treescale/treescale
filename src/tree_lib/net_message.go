package tree_lib

import (
	"net"
	"errors"
	"encoding/binary"
	"fmt"
	"github.com/pquerna/ffjson/ffjson"
)

// Reading network message as a byte array,
// where first 4 bytes is a uint length for all message
func ReadMessage(conn *net.TCPConn) (msg []byte, err TreeError) {
	var (
		msg_len_byte 	= 	make([]byte, 4)
		msg_len				uint32
		rlen				int
	)
	err.From = FROM_READ_MESSAGE
	rlen, err.Err = conn.Read(msg_len_byte)
	if !err.IsNull() {
		return
	}

	if rlen != 4 {
		err.Err = errors.New("Data lenght reading error. Check API details")
		return
	}

	msg_len = binary.LittleEndian.Uint32(msg_len_byte)
	msg = make([]byte, int(msg_len))
	rlen, err.Err = conn.Read(msg)
	if !err.IsNull() {
		return
	}

	if rlen != int(msg_len) {
		err.Err = errors.New(fmt.Sprintf("Message length not equal to given length. Check API details. Given %d message length, but recieved %d", int(msg_len), rlen))
		return
	}

	return
}

func ReadJson(v interface{}, conn *net.TCPConn) (err TreeError) {
	var (
		msg_data	[]byte
	)
	err.From = FROM_READ_JSON
	msg_data, err = ReadMessage(conn)
	if err.IsNull() {
		return
	}

	err.Err = ffjson.Unmarshal(msg_data, v)
	return
}

func SendMessage(data []byte, conn *net.TCPConn) (int, error) {
	var (
		data_len 	= 	make([]byte, 4)
		send_data		[]byte
	)
	binary.LittleEndian.PutUint32(data_len, uint32(len(data)))
	send_data = append(data_len, data...)
	return conn.Write(send_data)
}

func SendJson(v interface{}, conn *net.TCPConn) (len int, err TreeError) {
	var (
		s_data []byte
	)
	err.From = FROM_SEND_JSON
	s_data, err.Err = ffjson.Marshal(v)
	if !err.IsNull() {
		return
	}

	len, err.Err = SendMessage(s_data, conn)
	return
}