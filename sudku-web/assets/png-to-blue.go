package main

import (
	"image"
	"image/color"
	"image/png"
	"log"
	"os"
	"path/filepath"
)

func main() {
ArgLoop:
  for _, arg := range os.Args[1:] {
    ext := filepath.Ext(arg)
    if ext != ".png" {
      log.Print("only PNGs allowed")
      continue
    }
    f := must(os.Open(arg))
    img := must(png.Decode(f))
    f.Close()

    rect := img.Bounds()
    newImg := image.NewRGBA(rect)
    for x := rect.Min.X; x < rect.Max.X; x++ {
      for y := rect.Min.Y; y < rect.Max.Y; y++ {
        c, ok := img.At(x, y).(color.NRGBA)
        if !ok {
          log.Print("Only NRGBA colors supported")
          continue ArgLoop
        }
        if sum := c.R+c.G+c.B; sum == 0 {
          newImg.Set(x, y, color.NRGBA{R: 0, G: 255, B: 255, A: c.A})
        } else {
          newImg.Set(x, y, c)
        }
      }
    }

    nf := must(os.Create(arg[:len(arg)-4] + "-blue.png"))
    if err := png.Encode(nf, newImg); err != nil {
      log.Fatal(err)
    }
    nf.Close()
  }
}

func must[T any](t T, err error) T {
  if err != nil {
    log.Fatal(err)
  }
  return t
}
