package main

import (
	"fmt"
	"golang-whep-player/media"
	"golang-whep-player/whep"
)

func main() {
	client, err := whep.NewClient("h265")
	if err != nil {
		panic(err)
	}
	// sessionId := "073fb64a-63b3-4499-b2fa-13f78fa03dd2"
	// whepURL := "http://localhost:8080/whep?id=" + sessionId
	whepURL := "https://whep.vdo.ninja/kkkkeeee"

	client.OnDecoderReady = func(decoder media.Decoder) {
		fmt.Println("Decoder is ready. Streaming to ffplay.")
	}

	err = client.ConnectToWHEP(whepURL)
	if err != nil {
		fmt.Println("❌ Failed to connect to WHEP:", err)
		return
	}

	select {}
}
