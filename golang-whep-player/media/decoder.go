package media

import (
	"fmt"
	"io"
	"os/exec"
	"sync"

	"github.com/bluenviron/gortsplib/v4/pkg/format/rtph264"
	"github.com/bluenviron/gortsplib/v4/pkg/format/rtph265"
	"github.com/pion/rtp"
)

type Decoder interface {
	PushRTPPacket(pkt *rtp.Packet) error
	Close()
	FFmpegOut() io.Reader
}

// Shared FFmpeg Pipe Player
type ffmpegPipePlayer struct {
	ffmpegCmd *exec.Cmd
	ffmpegIn  io.WriteCloser
	mu        sync.Mutex
}

func startFFplay(codec string) (*ffmpegPipePlayer, error) {
	ffmpegCmd := exec.Command("ffmpeg",
		"-hide_banner", "-loglevel", "error",
		"-f", codec, "-i", "pipe:0",
		"-s", "1280x720",
		"-f", "rawvideo", "-pix_fmt", "yuv420p",
		"-")

	ffplayCmd := exec.Command("ffplay",
		"-f", "rawvideo",
		"-pixel_format", "yuv420p",
		"-video_size", "1280x720",
		"-")

	ffmpegOut, err := ffmpegCmd.StdoutPipe()
	if err != nil {
		return nil, fmt.Errorf("stdout pipe error: %v", err)
	}

	ffmpegIn, err := ffmpegCmd.StdinPipe()
	if err != nil {
		return nil, fmt.Errorf("stdin pipe error: %v", err)
	}

	ffplayCmd.Stdin = ffmpegOut

	if err := ffmpegCmd.Start(); err != nil {
		return nil, fmt.Errorf("failed to start ffmpeg: %v", err)
	}

	if err := ffplayCmd.Start(); err != nil {
		return nil, fmt.Errorf("failed to start ffplay: %v", err)
	}

	return &ffmpegPipePlayer{
		ffmpegCmd: ffmpegCmd,
		ffmpegIn:  ffmpegIn,
	}, nil
}

func (p *ffmpegPipePlayer) writeNALU(nalu []byte) error {
	p.mu.Lock()
	defer p.mu.Unlock()
	_, err := p.ffmpegIn.Write(nalu)
	return err
}

func (p *ffmpegPipePlayer) close() {
	p.ffmpegIn.Close()
	p.ffmpegCmd.Wait()
}

type H265Decoder struct {
	decoder *rtph265.Decoder
	player  *ffmpegPipePlayer
}

func NewH265Decoder() (Decoder, error) {
	player, err := startFFplay("hevc")
	if err != nil {
		return nil, err
	}

	decoder := &rtph265.Decoder{}
	decoder.Init()

	return &H265Decoder{
		decoder: decoder,
		player:  player,
	}, nil
}

func (d *H265Decoder) PushRTPPacket(pkt *rtp.Packet) error {
	nalus, err := d.decoder.Decode(pkt)
	if err != nil {
		fmt.Printf("❌ Decode error | Seq: %d | TS: %d | Err: %v\n", pkt.SequenceNumber, pkt.Timestamp, err)
		return nil
	}

	for _, nalu := range nalus {
		if len(nalu) == 0 {
			continue
		}

		const startCode = "\x00\x00\x00\x01"
		naluWithStartCode := append([]byte(startCode), nalu...)
		nalType := (nalu[0] >> 1) & 0x3F
		fmt.Printf("📦 H265 NALU Ready | Seq: %d | TS: %d | NALU Type: %d | Size: %d bytes\n",
			pkt.SequenceNumber, pkt.Timestamp, nalType, len(naluWithStartCode))

		if err := d.player.writeNALU(naluWithStartCode); err != nil {
			fmt.Printf("❌ Write to FFmpeg error: %v\n", err)
		}
	}
	return nil
}

func (d *H265Decoder) Close() {
	d.player.close()
}

func (d *H265Decoder) FFmpegOut() io.Reader {
	return nil
}

type H264Decoder struct {
	decoder *rtph264.Decoder
	player  *ffmpegPipePlayer
}

func NewH264Decoder() (Decoder, error) {
	player, err := startFFplay("h264")
	if err != nil {
		return nil, err
	}

	decoder := &rtph264.Decoder{}
	decoder.Init()

	return &H264Decoder{
		decoder: decoder,
		player:  player,
	}, nil
}

func (d *H264Decoder) PushRTPPacket(pkt *rtp.Packet) error {
	nalus, err := d.decoder.Decode(pkt)
	if err != nil {
		fmt.Printf("❌ Decode error | Seq: %d | TS: %d | Err: %v\n", pkt.SequenceNumber, pkt.Timestamp, err)
		return nil
	}

	for _, nalu := range nalus {
		if len(nalu) == 0 {
			continue
		}

		const startCode = "\x00\x00\x00\x01"
		naluWithStartCode := append([]byte(startCode), nalu...)
		nalType := nalu[0] & 0x1F
		fmt.Printf("📦 H264 NALU Ready | Seq: %d | TS: %d | NALU Type: %d | Size: %d bytes\n",
			pkt.SequenceNumber, pkt.Timestamp, nalType, len(naluWithStartCode))

		if err := d.player.writeNALU(naluWithStartCode); err != nil {
			fmt.Printf("❌ Write to FFmpeg error: %v\n", err)
		}
	}
	return nil
}

func (d *H264Decoder) Close() {
	d.player.close()
}

func (d *H264Decoder) FFmpegOut() io.Reader {
	return nil
}
