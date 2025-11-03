import net from "node:net";
import fs from "node:fs";
import path from "node:path";

const AUDIO_PATH = path.resolve("static/audio.pcm");
const PORT = 9000;
const TAG = "[TCP]"

function startTcpStreamer() {
    const server = net.createServer((socket) => {
            console.log(`${TAG} Device connected: ${socket.remoteAddress}`);

        const file = fs.createReadStream(AUDIO_PATH);
        file.pipe(socket);

        socket.on("close", () => {
            console.log(`${TAG} Device disconnected`);
            file.destroy();
        });

        socket.on("error", (err) => {
            console.error(`${TAG} Socket error: `, err);
        });
    });

    server.listen(PORT, "0.0.0.0", () => {
        console.log(`${TAG} TCP streamer listening on port: ${PORT}`);
    });
}

startTcpStreamer();