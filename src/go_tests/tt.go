package main
import (
	"cmd/compile/internal/big"
	"fmt"
)


func main() {
	a := big.NewInt(int64(-64))

	fmt.Println(a.Sign())
}