package main

import (
  "encoding/binary"
  "io/ioutil"
  "os"
  "fmt"
  "net"
  "time"
)

func main() {
  buffer, err := ioutil.ReadFile(os.Args[1])
  if err != nil {
    fmt.Println("Unable to open file -> ", err)
    return
  }

  send_buffer := make([]byte, 0)
  send_buffer2 := make([]byte, 0)

  api_version_buf := make([]byte, 4)
  data_len_buf := make([]byte, 4)
  token_info := []byte("test_api2|0")

  // appending api version and token info
  binary.BigEndian.PutUint32(api_version_buf, uint32(1))
  binary.BigEndian.PutUint32(data_len_buf, uint32(len(token_info)))
  send_buffer = append(send_buffer, api_version_buf...)
  send_buffer = append(send_buffer, data_len_buf...)
  send_buffer = append(send_buffer, token_info...)

  // adding event information
  ev_name := []byte("test_event")
  ev_from := []byte("test_api2")
  ev_data := buffer
  ev_buf_len := 4 + 2 + // path
                4 + len(ev_name) + // event name and length
                4 + len(ev_from) +
                4 + 0 + // target
                4 + 0 + //public data
                4 + len(ev_data) //event data

  // total len
  binary.BigEndian.PutUint32(data_len_buf, uint32(ev_buf_len))
  send_buffer2 = append(send_buffer2, data_len_buf...)

  // path
  binary.BigEndian.PutUint32(data_len_buf, uint32(2))
  send_buffer2 = append(send_buffer2, data_len_buf...)
  send_buffer2 = append(send_buffer2, []byte("25")...)

  // name
  binary.BigEndian.PutUint32(data_len_buf, uint32(len(ev_name)))
  send_buffer2 = append(send_buffer2, data_len_buf...)
  send_buffer2 = append(send_buffer2, ev_name...)

  // from
  binary.BigEndian.PutUint32(data_len_buf, uint32(len(ev_from)))
  send_buffer2 = append(send_buffer2, data_len_buf...)
  send_buffer2 = append(send_buffer2, ev_from...)

  // target
  binary.BigEndian.PutUint32(data_len_buf, uint32(0))
  send_buffer2 = append(send_buffer2, data_len_buf...)

  // public data
  binary.BigEndian.PutUint32(data_len_buf, uint32(0))
  send_buffer2 = append(send_buffer2, data_len_buf...)

  // data
  binary.BigEndian.PutUint32(data_len_buf, uint32(len(ev_data)))
  send_buffer2 = append(send_buffer2, data_len_buf...)
  send_buffer2 = append(send_buffer2, ev_data...)

  for i := 0; i < 150; i++ {
    go run_conn(send_buffer, send_buffer2)
  }

  for {
    time.Sleep(time.Second * 100)
  }
}

func run_conn(send_buffer, send_buffer2 []byte) {
  addr, err := net.ResolveTCPAddr("tcp", os.Args[2])
	if err != nil {
		fmt.Println("Unable to resolve address", err.Error())
		return
	}

	conn, err := net.DialTCP("tcp", nil, addr)
	if err != nil {
		fmt.Println("Unable to connect", err.Error())
		return
	}

  // just to free up socket buffer sent by echo server
	readable_buffer := make([]byte, 64)

  conn.Write(send_buffer)
  fmt.Println("data sent !")
  rsize, err := conn.Read(readable_buffer)
  fmt.Println(rsize)
  if err != nil {
    conn.Close()
    return
  }

  for {
    conn.Write(send_buffer2)
    time.Sleep(time.Millisecond * 100)
  }
}
