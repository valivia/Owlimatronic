import net from "node:net";
import fs from "node:fs";
import path from "node:path";

const TAG = "[STREAMER]";
const AUDIO_PATH = path.resolve("audio/stream.pcm");
const PORT = 9000; 

const server = net.createServer(socket => {
  console.log(`${TAG} Client connected:`, socket.remoteAddress);

  const file = fs.createReadStream(AUDIO_PATH);
  file.pipe(socket);

  file.on("end", () => {
    console.log(`${TAG} File done, closing...`);
    socket.end();
  });

  socket.on("error", e => console.error(`${TAG} Socket error:`, e));
});

server.listen(PORT, "0.0.0.0", () => console.log(`${TAG} Streaming on port`, PORT));