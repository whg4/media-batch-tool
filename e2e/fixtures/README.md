# E2E fixtures

Small media files used by the real-app E2E (`e2e/real-app.spec.ts`).

Generated with the system ffmpeg:

```bash
ffmpeg -y -loglevel error -f lavfi -i testsrc2=duration=0.1:size=1280x720:rate=1 -frames:v 1 e2e/fixtures/photo1.png
ffmpeg -y -loglevel error -f lavfi -i gradients=size=800x600 -frames:v 1 -q:v 5 e2e/fixtures/photo2.jpg
ffmpeg -y -loglevel error -f lavfi -i testsrc2=duration=2:size=640x360:rate=24 -f lavfi -i sine=frequency=440:duration=2 \
  -c:v libx264 -preset ultrafast -pix_fmt yuv420p -c:a aac -shortest e2e/fixtures/video1.mp4
```
