package whep

import (
	"bytes"
	"fmt"
	"io"
	"net/http"

	"golang-whep-player/media"

	"github.com/pion/webrtc/v4"
)

type WHEPClient struct {
	PeerConnection *webrtc.PeerConnection
	OnDecoderReady func(media.Decoder)
}

func NewClient(codecName string) (*WHEPClient, error) {
	m := &webrtc.MediaEngine{}

	switch codecName {
	case "h264":
		err := m.RegisterCodec(webrtc.RTPCodecParameters{
			RTPCodecCapability: webrtc.RTPCodecCapability{
				MimeType:  webrtc.MimeTypeH264,
				ClockRate: 90000,
			},
			PayloadType: 96,
		}, webrtc.RTPCodecTypeVideo)
		if err != nil {
			return nil, fmt.Errorf("register H264 failed: %v", err)
		}
	case "h265":
		err := m.RegisterCodec(webrtc.RTPCodecParameters{
			RTPCodecCapability: webrtc.RTPCodecCapability{
				MimeType:  webrtc.MimeTypeH265,
				ClockRate: 90000,
			},
			PayloadType: 102,
		}, webrtc.RTPCodecTypeVideo)
		if err != nil {
			return nil, fmt.Errorf("register H265 failed: %v", err)
		}
	default:
		return nil, fmt.Errorf("unsupported codec: %s", codecName)
	}

	api := webrtc.NewAPI(webrtc.WithMediaEngine(m))
	pc, err := api.NewPeerConnection(webrtc.Configuration{})
	if err != nil {
		return nil, fmt.Errorf("create PeerConnection failed: %v", err)
	}

	return &WHEPClient{PeerConnection: pc}, nil
}

func (c *WHEPClient) ConnectToWHEP(url string) error {
	c.PeerConnection.OnTrack(func(track *webrtc.TrackRemote, _ *webrtc.RTPReceiver) {
		fmt.Printf("[Track] Codec: %s | PayloadType: %d\n",
			track.Codec().MimeType, track.PayloadType())

		var err error
		var decoder media.Decoder
		switch track.Codec().MimeType {
		case webrtc.MimeTypeH264:
			decoder, err = media.NewH264Decoder()
		case webrtc.MimeTypeH265:
			decoder, err = media.NewH265Decoder()
		default:
			fmt.Println("Unsupported codec:", track.Codec().MimeType)
			return
		}
		if err != nil {
			fmt.Println("Decoder init error:", err)
			return
		}
		defer decoder.Close()

		if c.OnDecoderReady != nil {
			c.OnDecoderReady(decoder)
		}

		for {
			rtpPkt, _, err := track.ReadRTP()
			if err != nil {
				fmt.Println("Track read error:", err)
				break
			}
			if err := decoder.PushRTPPacket(rtpPkt); err != nil {
				fmt.Println("Decoder error:", err)
				break
			}
		}
	})

	_, err := c.PeerConnection.AddTransceiverFromKind(webrtc.RTPCodecTypeVideo,
		webrtc.RTPTransceiverInit{Direction: webrtc.RTPTransceiverDirectionRecvonly})
	if err != nil {
		return fmt.Errorf("add transceiver failed: %v", err)
	}

	offer, err := c.PeerConnection.CreateOffer(nil)
	if err != nil {
		return fmt.Errorf("CreateOffer failed: %v", err)
	}

	if err := c.PeerConnection.SetLocalDescription(offer); err != nil {
		return fmt.Errorf("SetLocalDescription failed: %v", err)
	}

	<-webrtc.GatheringCompletePromise(c.PeerConnection)

	resp, err := http.Post(url, "application/sdp",
		bytes.NewReader([]byte(c.PeerConnection.LocalDescription().SDP)))
	if err != nil {
		return fmt.Errorf("POST to WHEP failed: %v", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK && resp.StatusCode != http.StatusCreated {
		body, _ := io.ReadAll(resp.Body)
		return fmt.Errorf("WHEP server error: %s\n%s", resp.Status, string(body))
	}

	answerSDP, err := io.ReadAll(resp.Body)
	if err != nil {
		return fmt.Errorf("read answer SDP failed: %v", err)
	}

	answer := webrtc.SessionDescription{
		Type: webrtc.SDPTypeAnswer,
		SDP:  string(answerSDP),
	}

	if err := c.PeerConnection.SetRemoteDescription(answer); err != nil {
		return fmt.Errorf("SetRemoteDescription failed: %v", err)
	}

	fmt.Println("✅ WHEP connection established.")
	return nil
}
