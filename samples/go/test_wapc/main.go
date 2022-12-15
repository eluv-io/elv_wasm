package main

import (
	"strconv"

	wapc "github.com/wapc/wapc-guest-tinygo"
)

func main() {
	wapc.RegisterFunctions(wapc.Functions{
		"echo":      echo,
		"factorial": factorial,
		"crash_div": crash_div,
	})
}

func echo(bi []byte) ([]byte, error) {
	return bi, nil
}

func factorial(bi []byte) ([]byte, error) {
	factVal := uint64(1)
	sin := string(bi)

	n, err := strconv.Atoi(sin)
	if err != nil {
		return nil, err
	}
	for i := uint64(1); i <= uint64(n); i++ {
		factVal *= i
	}
	s1 := strconv.FormatUint(factVal, 10)
	return []byte(s1), nil
}

func crash_div(bi []byte) ([]byte, error) {
	i := 0
	_ = 5000 / i

	return nil, nil
}
