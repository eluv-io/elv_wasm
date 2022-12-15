package main

import (
	"fmt"
	"strconv"

	wapc "github.com/wapc/wapc-guest-tinygo"
)

func main() {
	wapc.RegisterFunctions(wapc.Functions{
		"echo":      echo,
		"factorial": factorial,
	})
}

func echo(bi []byte) ([]byte, error) {
	return bi, nil
}

func factorial(bi []byte) ([]byte, error) {
	factVal := uint64(1)
	sin := string(bi)
	wapc.ConsoleLog(fmt.Sprintf("input = %s", sin))

	n, err := strconv.Atoi(sin)
	if err != nil {
		return nil, err
	}
	wapc.ConsoleLog(fmt.Sprintf("Atoi = %d %d", n, uint64(n)))
	for i := uint64(1); i <= uint64(n); i++ {
		factVal *= i // mismatched types int64 and int
	}
	wapc.ConsoleLog(fmt.Sprintf("fact = %d", factVal))
	s1 := strconv.FormatUint(factVal, 10)
	wapc.ConsoleLog(fmt.Sprintf("s = %s", s1))
	return []byte(s1), nil
}
