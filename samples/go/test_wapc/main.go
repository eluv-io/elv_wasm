package main

import (
	b64 "encoding/base64"
	"strconv"

	wapc "github.com/wapc/wapc-guest-tinygo"
)

func main() {
	wapc.RegisterFunctions(wapc.Functions{
		"echo":         echo,
		"echo_sleep":   echo_sleep,
		"factorial":    factorial,
		"crash_div":    crash_div,
		"memory_check": memory_check,
	})
}

func echo(bi []byte) ([]byte, error) {
	return bi, nil
}

func echo_sleep(bi []byte) ([]byte, error) {
	wapc.HostCall("sleep", "5000", "", []byte{})
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

func memory_check(bi []byte) ([]byte, error) {
	s := b64.StdEncoding.EncodeToString(bi)
	return wapc.HostCall("foo", "", "", []byte(s))
}
