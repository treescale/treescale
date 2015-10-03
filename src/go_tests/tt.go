package main
import (
	"cmd/compile/internal/big"
	"fmt"
)


func main() {
	a := big.Int{}
	a.SetString("150", 10)
	a.Mod(big.NewInt(17), big.NewInt(3))
	fmt.Println(a.Int64())
}